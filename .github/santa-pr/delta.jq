# delta.jq — a pull request's two SANTA results compared: SANTA's PR check (pr-check/README.md), copied into an
# implementation's repository and run by its own workflow, so no SANTA code runs there.
#
#   jq -n -f delta.jq --slurpfile base base/results.json --slurpfile head head/results.json \
#     --arg label owner/repo --arg base_sha <sha> --arg head_sha <sha> --arg base_status "" --arg head_status ""
#
# Each side is a runner's results.json (santa-runner-results/v1, SANTA's runner-integration §4.3), or an empty file
# for a side that did not run; its status is "" when it ran, else why it could not run. An entry is (slice, file,
# entry); it is compared by its SET of red dimensions, never by the expected/got strings, which are for people:
# newly red, now green, red another way (changed), or still red. A file graded on one side only is listed apart,
# with its red entries there.
# Writes delta.json (santa-pr-delta/v1); comment.jq renders it.

# a results.json this program and comment.jq can read: santa-runner-results/v1 with every field either of them
# uses, of the type it uses it as. The single definition: the workflow only checks a side's file is JSON at all, and
# anything else unreadable is a side that could not run, never a crash.
def strings: type == "array" and all(.[]; type == "string");
def readable:
  type == "object" and .schema == "santa-runner-results/v1" and (.santa | type == "string" or type == "null")
  and (.scope | type == "object" and (.version | type) == "string" and (.tiers | strings)
       and (.cost | type) == "boolean" and (.kinds | . == null or strings)
       and (.activation | . == null or type == "boolean"))
  and (.slices | type == "object"
       and all(.[]; type == "object" and (.files | strings) and (.red | type) == "array"
               and all(.red[]; type == "object" and (.file | type) == "string" and (.entry | type) == "string"
                       and (.dim | type) == "string" and (.note | . == null or type == "string"))));

# the side as given, or why it could not be compared
def side($r; $status):
  if $status != "" then {why: $status}
  elif ($r | length) == 0 then {why: "no results.json"}
  elif ($r[0] | readable | not) then {why: "results.json is not santa-runner-results/v1"}
  else {r: $r[0]}
  end;

# the declared scope as SANTA's conform prints it
def scope_text:
  "version≤\(.version), tiers=\(.tiers | join(",")), cost=\(.cost)"
  + (if .kinds then ", kinds=\(.kinds | join(","))" else "" end)
  + (if .activation == false then ", activation=false" else "" end);

# every graded file of a side
def files: [.slices[] | .files[]] | unique;

# a side's red entries: "slice\u0000file\u0000entry" -> {slice, file, entry, dims: {dim: {expected, got}}, note?}
def reds:
  reduce (.slices | to_entries[] | .key as $slice | .value.red[] | . + {slice: $slice}) as $x ({};
    ("\($x.slice)\u0000\($x.file)\u0000\($x.entry)") as $k
    | .[$k].slice = $x.slice | .[$k].file = $x.file | .[$k].entry = $x.entry
    | .[$k].dims[$x.dim] = {expected: $x.expected, got: $x.got}
    | if $x.note and (.[$k].note | not) then .[$k].note = $x.note else . end);

def short: .[0:7];

# a set of strings as an object, for lookups
def set: map({key: ., value: true}) | from_entries;

# a side's red entries per file
def red_counts: [.[] | .file] | group_by(.) | map({key: .[0], value: length}) | from_entries;

side($base; $base_status) as $b
| side($head; $head_status) as $h
| {schema: "santa-pr-delta/v1", label: $label, base_sha: $base_sha, head_sha: $head_sha,
   santa: (if $b.r and $h.r then (if $b.r.santa == $h.r.santa then $h.r.santa else null end)
           else ($h.r.santa // $b.r.santa // null) end),
   scope_changed: null, files: {only_base: [], only_head: []},
   newly_red: [], now_green: [], changed: [], still_red: 0}
| if $b.why and $h.why then . + {outcome: "not-compared", reason: "neither side could run: base: \($b.why); head: \($h.why)"}
  elif $b.why then . + {outcome: "not-compared", reason: "base could not run: \($b.why)"}
  elif $h.why then . + {outcome: "not-compared", reason: "head could not run: \($h.why)"}
  elif $b.r.santa != $h.r.santa then
    . + {outcome: "not-compared",
         reason: "graded against different SANTA commits: \($b.r.santa | short) and \($h.r.santa | short)"}
  else
    . as $d
    | ($b.r | files) as $bf | ($h.r | files) as $hf
    | ($bf - $hf) as $only_base | ($hf - $bf) as $only_head
    | ($b.r | reds) as $br | ($h.r | reds) as $hr
    | ($only_base + $only_head | set) as $apart
    # entries of files graded on both sides only
    | [($br + $hr) | to_entries[] | select($apart[.value.file] | not) | .key] | unique as $keys
    | [$keys[] as $k
       | {slice: ($hr[$k].slice // $br[$k].slice), file: ($hr[$k].file // $br[$k].file),
          entry: ($hr[$k].entry // $br[$k].entry),
          base: ($br[$k].dims // {}), head: ($hr[$k].dims // {})}
         + (if ($hr[$k].note // $br[$k].note) then {note: ($hr[$k].note // $br[$k].note)} else {} end)]
      | sort_by(.slice, .file, .entry) as $all
    | $d + {outcome: "compared",
            scope_changed: (($b.r.scope | scope_text) as $sb | ($h.r.scope | scope_text) as $sh
                            | if $sb != $sh then {base: $sb, head: $sh} else null end),
            files: (($br | red_counts) as $bc | ($hr | red_counts) as $hc
                    | {only_base: [$only_base[] | {file: ., red_entries: ($bc[.] // 0)}],
                       only_head: [$only_head[] | {file: ., red_entries: ($hc[.] // 0)}]}),
            newly_red: [$all[] | select((.base | length) == 0 and (.head | length) > 0)],
            now_green: [$all[] | select((.base | length) > 0 and (.head | length) == 0)],
            changed: [$all[] | select((.base | length) > 0 and (.head | length) > 0
                                      and (.base | keys) != (.head | keys))],
            still_red: ([$all[] | select((.base | length) > 0 and (.base | keys) == (.head | keys))] | length)}
  end
