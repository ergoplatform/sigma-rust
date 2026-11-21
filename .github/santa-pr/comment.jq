# comment.jq — delta.json (delta.jq's output) as markdown: SANTA's PR check (pr-check/README.md), copied into an
# implementation's repository and run by its own workflow.
#
#   jq -r -f comment.jq --argjson cap 50   delta.json > comment.md     # the comment: at most 50 files per section
#   jq -r -f comment.jq --argjson cap null delta.json >> "$GITHUB_STEP_SUMMARY"   # the job summary: every file, then
#                                                                                  # every entry with its answers
#
# The layout is SANTA's santa-delta's: the counts, newly red and now green one row per vector file (the largest
# first), red another way folded, still red counted, files graded on one side only with their red entries. Every
# cell is escaped, so a name cannot break the table or open a tag. The comment is cut at 60,000 characters (GitHub's
# limit is 65,536), the job summary at 1,000,000 bytes (its limit is 1 MiB). Written for jq 1.7 and 1.8 alike: a binding inside
# an operand (`A + (B as $x | C)`) is always parenthesized, because jq 1.8 binds `A + B as $x | C` as `(A + B) as $x`.

# a cell's text, inert: no tag (<, >, &, "), no table split (|), no markdown (* _ ` [ ] ~ \\), no @mention (the text
# comes from the pull request's own artifact, and the bot would notify the account), on one line. One pass.
def escapes: {"&": "&amp;", "<": "&lt;", ">": "&gt;", "\"": "&quot;", "|": "&#124;", "@": "&#64;", "\\": "&#92;",
  "`": "&#96;", "*": "&#42;", "_": "&#95;", "[": "&#91;", "]": "&#93;", "~": "&#126;", "\n": " ", "\r": " "};
def esc: gsub("(?<c>[&<>\"|@\\\\`*_\\[\\]~\n\r])"; escapes[.c]);
def code: "<code>\(esc)</code>";
def brief: if type == "string" then . else tojson end;
def plural($one; $many): "\(.) \(if . == 1 then $one else $many end)";
def short: if . == null or . == "" then "—" else "`\(.[0:7])`" end;
def dimset: keys | join("+");
def side_text: to_entries | sort_by(.key)
  | map("\(.key | esc): expected \(.value.expected | brief | code), got \(.value.got | brief | code)") | join("; ");

# a section's key in delta.json, and its name
def field($s): {newly: "newly_red", green: "now_green", changed: "changed"}[$s];
def title($s): {newly: "🪨 Newly red", green: "🎁 Now green", changed: "↔ Red another way"}[$s];

# a section's changes, one row per vector file: the largest first, its split (by dimension, or "base → head") too
def rows($s):
  group_by([.slice, .file])
  | map({slice: .[0].slice, file: .[0].file, entries: length,
         split: ([.[] | if $s == "newly" then (.head | keys[]) elif $s == "green" then (.base | keys[])
                        else "\(.base | dimset) → \(.head | dimset)" end]
                 | group_by(.) | map({k: .[0], n: length}) | sort_by(-.n, .k))})
  | sort_by(-.entries, .slice, .file);

def heading($d; $s):
  ($d[field($s)]) as $ch
  | "\(title($s)): \($ch | length | plural("entry"; "entries")) in \($ch | rows($s) | length | plural("file"; "files"))";

# a section's file table, at most $cap rows (null: all), the rest summed in one line
def table($s; $cap):
  rows($s) as $rs
  | (if $cap == null then ($rs | length) else $cap end) as $n
  | "| slice | file | entries | \(if $s == "changed" then "change" else "dimensions" end) |\n|---|---|---:|---|\n"
    + ([$rs[0:$n][] | "| \(.slice | code) | \(.file | code) | \(.entries) | \([.split[] | "\(.k | esc) \(.n)"] | join(" · ")) |\n"]
       | join(""))
    + (if ($rs | length) > $n then
         $rs[$n:] as $rest
         | "\n_… \($rest | length) more \(if ($rest | length) == 1 then "file" else "files" end) (\([$rest[].entries] | add | plural("entry"; "entries"))): see the job summary._\n"
       else "" end);

# the files graded on one side only, with their red entries there, at most $cap rows (null: all)
def files_table($title; $files; $cap):
  if ($files | length) == 0 then ""
  else
    (if $cap == null then ($files | length) else $cap end) as $n
    | "### \($title)\n\n| file | red entries |\n|---|---:|\n"
      + ([$files[0:$n][] | "| \(.file | code) | \(.red_entries) |\n"] | join(""))
      + (if ($files | length) > $n then
           $files[$n:] as $rest
           | "\n_… \($rest | length) more \(if ($rest | length) == 1 then "file" else "files" end) (\([$rest[].red_entries] | add | plural("red entry"; "red entries"))): see the job summary._\n"
         else "" end)
      + "\n"
  end;

def block($cap):
  . as $d
  | "## 🎅 SANTA · \($d.label | esc)\n\nbase \($d.base_sha | short) → head \($d.head_sha | short)\n\n"
    + (if $d.outcome != "compared" then "⚠️ No delta: \($d.reason | esc)\n\n"
       else
         (if $d.scope_changed then
            "**Scope changed:** base \($d.scope_changed.base | code) → head \($d.scope_changed.head | code)\n\n"
          else "" end)
         + ((([$d.newly_red, $d.now_green, $d.changed] | map(length) | add) > 0) as $moved
         | (($d.files.only_base + $d.files.only_head | length) > 0) as $files_moved
         | if ($moved | not) and ($files_moved | not) then
             "No change: \($d.still_red | plural("entry"; "entries")) still red, as at base.\n\n"
           else
             "| 🪨 newly red | 🎁 now green | ↔ red another way | still red |\n|---:|---:|---:|---:|\n"
             + "| \($d.newly_red | length) | \($d.now_green | length) | \($d.changed | length) | \($d.still_red) |\n\n"
             + ([("newly", "green") as $s | select(($d[field($s)] | length) > 0)
                 | "### \(heading($d; $s))\n\n\($d[field($s)] | table($s; $cap))\n"] | join(""))
             + files_table("Graded at head only (not at base)"; $d.files.only_head; $cap)
             + files_table("Graded at base only (not at head)"; $d.files.only_base; $cap)
             + (if ($d.changed | length) > 0 then
                  "<details><summary>\(heading($d; "changed"))</summary>\n\n\($d.changed | table("changed"; $cap))\n</details>\n\n"
                else "" end)
           end)
       end);

# the job summary's tail: every entry of a section, grouped by file, with its answers
def per_entry:
  . as $d
  | [("newly", "changed", "green") as $s
     | ($d[field($s)]) as $ch
     | select(($ch | length) > 0)
     | "### \(heading($d; $s)), per entry\n\n"
       # grouped by file (delta.json lists them sorted by slice, file, entry), joined once: linear in the entries
       + ([$ch | group_by([.slice, .file])[]
           | "\n**\(.[0].slice | code) · \(.[0].file | code)**\n\n"
             + ([.[] as $c
                 | "- \($c.entry | code): \(if $s == "newly" then ($c.head | side_text)
                                             elif $s == "green" then "was \($c.base | side_text)"
                                             else "base \($c.base | side_text) → head \($c.head | side_text)" end)\n"]
                | join(""))]
          | join(""))
       + "\n"]
  | join("");

# the markdown cut at whole lines to $limit, the cut line last: in characters for a comment (GitHub counts those),
# in UTF-8 bytes for the job summary (its limit is 1 MiB)
def cut($limit; $line; $bytes):
  def size: if $bytes then utf8bytelength else length end;
  if size <= $limit then .
  else
    split("\n") as $lines
    | [foreach $lines[] as $l (0; . + ($l | size) + 1)] as $ends
    | ([$ends | to_entries[] | select(.value + ($line | size) + 2 > $limit) | .key] | first // ($lines | length)) as $n
    | ($lines[0:$n] | join("\n")) + "\n\n" + $line
  end;

(block($cap) + (if $cap == null and .outcome == "compared" then per_entry else "" end))
| if $cap == null then cut(1000000; "_… cut at 1,000,000 bytes: every entry is in delta.json._"; true)
  else cut(60000; "_… cut at 60,000 characters: see the job summary._"; false) end
| sub("\n+$"; "")
