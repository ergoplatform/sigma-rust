from typing import Union

from ergo_lib_python.chain import Address, ErgoBoxCandidate
from ergo_lib_python.ergo_tree import ErgoTree

Script = Union[Address, ErgoTree]


def accepts_valid(script: Script) -> ErgoBoxCandidate:
    return ErgoBoxCandidate(value=1_000_000, script=script, creation_height=0)


def rejects_invalid(script: Script) -> None:
    ErgoBoxCandidate(bogus=True)  # type: ignore[call-arg]
    ErgoBoxCandidate(value="bad", script=script, creation_height="bad")  # type: ignore[arg-type]
    ErgoBoxCandidate(1, script, 0)  # type: ignore[misc]
