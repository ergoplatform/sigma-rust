"""On-chain types"""
import typing
from typing import Self, Optional, cast
from ergo_lib_python import transaction
from ergo_lib_python.sigma_protocol import ProveDlog
from ergo_lib_python import ergo_tree
__all__ = ['NetworkPrefix', 'Address', 'EcPoint', 'ErgoBoxCandidate', 'ErgoBox', 'BoxId', 'TokenId', 'Token', 'NonMandatoryRegisterId', 'NonMandatoryRegisters', 'Constant', 'SType', 'BlockId', 'Header', 'PreHeader', 'ContextExtension', 'Parameters', 'ErgoStateContext']
# Types are written as 'SType' here because SType can't be named until after its definition exits. Also using SType directly causes sphinx-autoapi to error due to recursion limit
# sphinx-autoapi unfortunately recognizes type of base class as str, so we need to manually override __init__ and class documentation to avoid confusion
class SType:
    """Type of a :py:class:`Constant`"""
    @typing.final
    class SUnit('SType'):
        """Unit type"""
        __match_args__ = ()
        def __init__(self, *args):
            """Can be built using ``SType.SUnit()``"""

        ...
    @typing.final
    class SBoolean('SType'):
        __match_args__ = ()
        """Boolean type"""
        def __init__(self, *args):
            "Can be built using ``SType.SBoolean()``"
        ...
    @typing.final
    class SByte('SType'):
        __match_args__ = ()
        """8-bit signed integer type"""
        def __init__(self, *args):
            "Can be built using ``SType.SByte()``"
        ...
    @typing.final
    class SShort('SType'):
        __match_args__ = ()
        """16-bit signed integer type"""
        def __init__(self, *args):
            "Can be built using ``SType.SShort()``"
        ...
        ...
    @typing.final
    class SInt('SType'):
        __match_args__ = ()
        """32-bit signed integer type"""
        def __init__(self, *args):
            "Can be built using ``SType.SInt()``"
        ...
    @typing.final
    class SLong('SType'):
        __match_args__ = ()
        """64-bit signed integer type"""
        def __init__(self, *args):
            "Can be built using ``SType.SLong()``"
        ...
    @typing.final
    class SBigInt('SType'):
        __match_args__ = ()
        """256-bit signed integer type"""
        def __init__(self, *args):
            "Can be built using ``SType.SBigInt()``"
        ...
    @typing.final
    class SUnsignedBigInt('SType'):
        __match_args__ = ()
        """256-bit unsigned  integer type"""
        def __init__(self, *args):
            "Can be built using ``SType.SUnsignedBigInt()``"
        ...
    @typing.final
    class SGroupElement('SType'):
        __match_args__ = ()
        """Elliptic curve point type"""
        def __init__(self, *args):
            "Can be built using ``SType.SGroupElement()``"
        ...
    @typing.final
    class SSigmaProp('SType'):
        __match_args__ = ()
        """SigmaProp type"""
        def __init__(self, *args):
            "Can be built using ``SType.SSigmaProp()``"
        ...
    @typing.final
    class SBox('SType'):
        __match_args__ = ()
        """ErgoBox type"""
        def __init__(self, *args):
            "Can be built using ``SType.SBox()``"
        ...
    @typing.final
    class SAvlTree('SType'):
        __match_args__ = ()
        """AvlTree type"""
        def __init__(self, *args):
            "Can be built using ``SType.SAvlTree()``"
        ...
    @typing.final
    class SHeader:
        __match_args__ = ()
        """Header type. Note that creating :py:class:`Constant` with :py:class`Header` value won't be possible until v6.0 soft fork"""

    @typing.final
    class SString:
        __match_args__ = ()
        """String type. Shouldn't be used, use SType.SColl(SType.SByte) instead"""
    @typing.final
    class SOption:
        __match_args__ = ('_0',)
        """Optional type"""
        def __init__(self, _0: 'SType'):
            "Can be built using ``SType.SOption(inner_stype)``"
        @property
        def _0(self) -> 'SType':
            "Get inner type of Option"
            ...
        ...
    @typing.final
    class SColl:
        __match_args__ = ('_0', )
        """Collection type (all items in the collection have same type)"""
        def __init__(self, _0: 'SType'):
            "Can be built using ``SType.SColl(elem_tpe)``"
        @property
        def _0(self) -> 'SType':
            "Get inner type of Coll"
            ...
        ...
    @typing.final
    class STuple:
        __match_args__ = ('_0',)
        """Tuple of heterogenous types

           Example:
               >>> c = Constant((Constant(True), Constant.from_i32(2)))
               >>> assert c.tpe == SType.STuple((SType.SBoolean(), SType.SInt()))
        """
        @property
        def _0(self) -> tuple['SType', ...]:
            """Get tuple of types"""
            ...
        def __init__(self, args: tuple['SType', ...]):
            ...

@typing.final
class EcPoint:
    """Elliptic curve point"""
    def __init__(self, b: bytes):
        ":param b: compressed (33 byte) array"
        ...
    def __bytes__(self) -> bytes:
        """Return EcPoint as compressed bytes"""
        ...
    ...

@typing.final
class Constant:
    """
    Constant value that can be used in ErgoTree, as a register, and in ContextExtension.
    Constants can be built from byte-arrays, integers, ErgoBox, collections of constants, etc. For the full list see parameter description

    """
    def __init__(self, arg: bytes | EcPoint | bool | ErgoBox | list[Self] | tuple[Self, ...], elem_tpe: Optional[SType] = None):
        """
        Example:
            >>> # Build a signed 32-bit integer Constant
            >>> c = Constant.from_i32(2)
            >>> assert c.value == 2
            >>> #Build an array of 64-bit Constants
            >>> c2 = Constant([Constant.from_i64(1), Constant.from_i64(2)])
            >>> assert c2.value == [Constant.from_i64(1), Constant.from_i64(2)]
            >>> # Build a tuple (items can be heterogenous)
            >>> c3 = Constant((Constant.from_i8(1), Constant(True), c2, Constant(bytes([1, 2, 3]))))
            >>> assert c3.tpe == SType.STuple((SType.SByte(), SType.SBoolean(), SType.SColl(SType.SLong()), SType.SColl(SType.SByte())))

        Integers in Constants have a fixed bit width. To build a Constant from an int see Constant.from_i8, from_i16, from_i32, etc:
            >>> # These are the same size, and are equivalent
            >>> assert Constant.from_i32(2) == Constant.from_i32(2)
            >>> # These have different types, so they are not equal
            >>> assert Constant.from_i32(2) != Constant.from_i64(2)

        :param arg: value to build constant from

        :param elem_tpe: Explicit type used for building collections of constants when type can not be inferred (list is empty)

        """
        ...
    @classmethod
    def from_u256(cls, v: int) -> Self:
        """Create a 256-bit unsigned integer ``Constant`` (``SType.SUnsignedBigInt()``)

           | UnsignedBigInts are a v6.0 soft-fork type and can't be used as a register or
           | :py:class:`ContextExtension` value. They are only allowed as constants in :py:class:`ErgoTree`'s with version >= 3
        """
        ...
    @classmethod
    def from_i256(cls, v: int) -> Self:
        """Create a 256-bit signed integer ``Constant`` (``SType.SBigInt()``) """
        ...
    @classmethod
    def from_i64(cls, v: int) -> Self:
        """Create a 64-bit signed integer ``Constant`` (``SType.SLong()``) """
        ...
    @classmethod
    def from_i32(cls, v: int) -> Self:
        """Create a 32-bit signed integer ``Constant`` (``SType.SInt()``) """
        ...
    @classmethod
    def from_i16(cls, v: int) -> Self:
        """Create a 16-bit signed integer ``Constant`` (``SType.SShort()``) """
        ...
    @classmethod
    def from_i8(cls, v: int) -> Self:
        """Create a 16-bit signed integer ``Constant`` (``SType.SByte()``) """
        ...
    @classmethod
    def from_bytes(cls, b: bytes) -> Self:
        """Parse Constant from byte array"""
        ...
    @property
    def tpe(self) -> SType:
        """Return type of Constant"""
        ...
    @property
    def value(self) -> bytes | EcPoint | bool | ErgoBox | list[Self] | tuple[Self]:
        ...
    def __bytes__(self) -> bytes:
        """Serialize Constant to byte array using ``bytes()``"""
        ...

@typing.final
class ContextExtension:
    """User-defined variables to be put into context. Keys are unsigned 8-bit integers and values are :py:class:`Constant`

    Example:
        >>> c = ContextExtension({0: Constant.from_i32(1), 1: Constant.from_i64(1)})
        >>> assert 0 in c
        >>> assert c[0] == Constant.from_i32(1)

    """

    def __init__(self, values: Optional[dict[int, Constant]] = None):
        ...
    def __len__(self) -> int:
        ...
    def __contains__(self, index: int) -> bool:
        ...
    def __getitem__(self, index: int) -> Constant:
        ...
    def __setitem__(self, index: int, value: Constant):
        ...
    def __bytes__(self) -> bytes:
        """Serialize ContextExtension to bytes"""
        ...

    ...
@typing.final
class NetworkPrefix:
    """Type of network that is prefixed to address"""
    Mainnet: Self = cast(Self, ...)
    Testnet: Self = cast(Self, ...)
    def __int__(self) -> int:
        ...

@typing.final
class Address:
    """
     An address is a short string corresponding to some script used to protect a box.

     | Unlike (string-encoded) binary representation of a script, an address has some useful characteristics:

      - Integrity of an address could be checked, as it is incorporating a checksum.
      - A prefix of address is showing network and an address type.
      - An address is using an encoding (namely, Base58) which is avoiding similarly l0Oking characters, friendly to double-clicking and line-breaking in emails.

     | An address is encoding network type, address type, checksum, and enough information to watch for a particular scripts.
     |
     | Possible network types are:

      - :py:class:`Mainnet<NetworkPrefix.Mainnet>` - 0x00
      - :py:class:`Testnet<NetworkPrefix.Testnet>` - 0x10

      For an address type, we form content bytes as follows:

      - P2PK - serialized (compressed) public key
      - P2SH - first 192 bits of the Blake2b256 hash of serialized script bytes
      - P2S  - serialized script

     | Address examples for testnet:

      - P2PK (3WvsT2Gm4EpsM9Pg18PdY6XyhNNMqXDsvJTbbf6ihLvAmSb7u5RN)
      - P2SH (rbcrmKEYduUvADj9Ts3dSVSG27h54pgrq5fPuwB)
      - P2S (Ms7smJwLGbUAjuWQ)

     | For mainnet:

      - P2PK (9fRAWhdxEsTcdb8PhGNrZfwqa65zfkuYHAMmkQLcic1gdLSV5vA)
      - P2SH (8UApt8czfFVuTgQmMwtsRBZ4nfWquNiSwCWUjMg)
      - P2S (4MQyML64GnzMxZgm, BxKBaHkvrTvLZrDcZjcsxsF7aSsrN73ijeFZXtbj4CXZHHcvBtqSxQ)
     """
    def __init__(self, arg: str | bytes, network_prefix: Optional[NetworkPrefix] = None):
        """Create address from string or bytes argument.
        By default this will accept both mainnet and testnet addresses. If you want to only allow a certain network address, set network_prefix

        :param arg: Base-58 encoded string or bytes
        :param network_prefix: Prefix of network the address belongs to. If set, then only addresses belonging to the specified network will be accepted, and an exception will be thrown if addresses from other networks are provided
        """
        ...

    @classmethod
    def p2pk(cls, prove_dlog: ProveDlog) -> Self:
        """Create a P2PK address from prove_dlog"""
        ...
    @classmethod
    def recreate_from_ergo_tree(cls, tree: ergo_tree.ErgoTree) -> Self:
        """Recreate address from ErgoTree

        At some point in the past a user entered an address from which the ErgoTree was built.
        Re-create the address from this `ErgoTree`.
        """
        ...
    def ergo_tree(self) -> ergo_tree.ErgoTree:
        """Creates an ErgoTree script from the address"""
        ...
    def to_str(self, network_prefix: NetworkPrefix) -> str:
        """Encode address as Base58 string

        :param network_prefix: Type of the network
        """
        ...

@typing.final
class BoxId:
    """Identifier of an :py:class:`ErgoBox`"""
    def __init__(self, val: str | bytes):
        """:param val: base-16 encoded str or raw bytes
        :raises ValueError: if val is not valid or not the correct length"""
    def __bytes__(self) -> bytes:
        ...
    def __str__(self) -> str:
        ...
    ...
@typing.final
class TokenId:
    """Identifier of a :py:class:`Token`"""
    def __init__(self, val: str | bytes):
        """:param val: base-16 encoded str or raw bytes
        :raises ValueError: if val is not valid or not the correct length"""
    @classmethod
    def from_box_id(cls, box_id: BoxId) -> Self:
        """Create a new TokenId from BoxId. When a new :py:class:`Token` is minted, its :py:class:`TokenId` must be equal to the id of the first input box in the transaction"""
    def __bytes__(self) -> bytes:
        ...

@typing.final
class Token:
    """Token represented with token id and amount"""
    def __init__(self, token_id: TokenId, amount: int):
        ...
    @property
    def token_id(self) -> TokenId:
        ...
    @property
    def amount(self) -> int:
        ...

@typing.final
class NonMandatoryRegisterId:
    """Additional :py:class:`ErgoBox` registers (R4-R9)"""
    R4: Self
    R5: Self
    R6: Self
    R7: Self
    R8: Self
    R9: Self
    def __int__(self) -> int:
        ...
@typing.final
class NonMandatoryRegisters:
    """Stores non-mandatory registers for :py:class:`ErgoBox` or :py:class:`ErgoBoxCandidate`. Immutable"""
    def __getitem__(self, index: NonMandatoryRegisterId) -> Constant:
        """:raises KeyError: if index does not exist"""
        ...
    def __len__(self) -> int:
        ...
    def __bytes__(self) -> bytes:
        """Serialize registers to byte representation"""


@typing.final
class ErgoBoxCandidate:
    """
    :py:class:`ErgoBox` candidate not yet included in any transaction on the chain
    """
    SAFE_USER_MIN: int = ... #Recommended (safe) minimal box value to use in case box size estimation is unavailable. Allows box size up to 2777 bytes with current min box value per byte of 360 nanoERGs
    """Contains the same fields as :py:class:`ErgoBox`, except for transaction id and index, that will be calculated after full transaction formation."""
    def __new__(cls, *,
        value: int,
        script: Address | ergo_tree.ErgoTree,
        creation_height: int, tokens: Optional[list[Token]] = None,
        registers: Optional[dict[NonMandatoryRegisterId, Constant]] = None,
        mint_token: Optional[Token] = None,
        mint_token_name: Optional[str] = None,
        mint_token_desc: Optional[str] = None,
        mint_token_decimals: Optional[int] = None) -> Self:
            """
                :param value: Amount of nanoERGs in the box
                :param script: Script that box is guarded by
                :param creation_height: Height that box is created on. This height is declared by the user and should not exceed height of the block
                :param tokens: List of tokens
                :param registers: Register values
                :param mint_token: Optional token to be minted
                :param mint_token_name: Name of token to be minted
                :param mint_token_desc: Description of token to be minted
                :param mint_token_decimals: Decimals of token to be minted
            """
            ...
    @property
    def value(self) -> int:
        """Amount of nanoErgs in the box"""
        ...
    @property
    def creation_height(self) -> int:
        """Creation height of box"""
        ...
    @property
    def tokens(self) -> list[Token]:
        """Tokens of box"""
        ...
    @property
    def additional_registers(self) -> NonMandatoryRegisters:
        """Additional registers of box"""
        ...
    @property
    def ergo_tree(self) -> ergo_tree.ErgoTree:
        """Script that box is guarded by"""
        ...


@typing.final
class ErgoBox:
    """
    Box (aka coin, or an unspent output) is a basic concept of a UTXO-based cryptocurrency.

    | In Bitcoin, such an object is associated with some monetary value (arbitrary,
    | but with predefined precision, so we use integer arithmetic to work with the value),
    | and also a guarding script (aka proposition) to protect the box from unauthorized opening.
    |
    | In other way, a box is a state element locked by some proposition (ErgoTree).
    |
    | In Ergo, a box is just a collection of registers, some with mandatory types and semantics, others could be used by applications in any way.
    | We add additional fields in addition to amount and proposition~(which is stored in the registers R0 and R1).
    | Namely, register R2 contains additional tokens (a sequence of pairs (token identifier, value)).
    | Register R3 contains height specified by user (protocol checks if it was <= current height when
    | transaction was accepted) and also transaction identifier and box index in the transaction outputs.
    | Registers R4-R9 are free for arbitrary usage.

    | A transaction is unsealing a box. As a box can not be opened twice, any further valid transaction can not be linked to the same box.
    """
    @classmethod
    def from_box_candidate(cls, candidate: ErgoBoxCandidate, tx_id: transaction.TxId, index: int) -> Self:
        ...
    @classmethod
    def from_json(cls, json: str | dict) -> Self:
        """| Deserialize :py:class:`ErgoBox` from json. For convenience json can be a :py:class:`dict` loaded from json.loads or similar functions
           | This can be useful when deserializing an array of boxes,
             as you can use json.loads to build a `list[dict]` and use :py:meth:`ErgoBox.from_json` on each item to convert to :py:class:`ErgoBox`

           .. code-block:: python

              import json
              boxes = [ErgoBox.from_json(box) for box in json.loads("[box1, box2, ...]")]
        """
        ...
    @classmethod
    def from_bytes(cls, b: bytes) -> Self:
        """Deserialize :py:class:`ErgoBox` from bytes

        :param b: bytes to parse
        :raises SigmaParsingException: if deserialization fails
        """
        ...
    @property
    def box_id(self) -> BoxId:
        """Unique identifier for box"""
        ...
    @property
    def value(self) -> int:
        """Value of box in nanoERG"""
        ...
    @property
    def creation_height(self) -> int:
        """Creation height of box

        This height is declared by user and should not exceed height of the block,
        """
        ...
    @property
    def tokens(self) -> list[Token]:
        """Tokens in box"""
        ...
    @property
    def additional_registers(self) -> NonMandatoryRegisters:
        """Additional registers in box

           :raises RegisterValueException: if parsing register value fails
        """

        ...
    @property
    def ergo_tree(self) -> ergo_tree.ErgoTree:
        """Script that box is guarded by"""
        ...
    @property
    def transaction_id(self) -> transaction.TxId:
        """Transaction that box was created in"""
        ...
    @property
    def index(self) -> int:
        """number of box (from 0 to total number of boxes the transaction with transactionId created - 1)"""
        ...
    def json(self) -> str:
        """Serialize to JSON"""
        ...
    def __bytes__(self) -> bytes:
        """Convert :py:class:`ErgoBox` to bytes"""
        ...

@typing.final
class BlockId:
    """Identifier of a :py:class:`Header`"""
    def __new__(cls, val: str | bytes):
        """:param val: base-16 encoded str or bytes"""
        ...
    def __bytes__(self) -> bytes:
        ...

@typing.final
class Header:
    """Represents data of the header available in sigma propositions
    """
    @classmethod
    def from_json(cls, json: dict | str) -> Self:
        """Can be created from str, or a dict which can be convenient when deserializing a list of Headers"""
        ...
    @property
    def version(self) -> int:
        """Version of header. Increases every fork"""
        ...
    @property
    def id(self) -> BlockId:
        """Identifier of block header"""
        ...
    @property
    def parent_id(self) -> BlockId:
        """Identifier of parent header"""
        ...
    @property
    def ad_proofs_root(self) -> bytes:
        """Hash of ADProofs for transactions in block"""
        ...
    @property
    def state_root(self) -> bytes:
        """AvlTree of a state after block application"""
        ...
    @property
    def transaction_root(self) -> bytes:
        """Merkle Tree root hash of transactions in a block"""
        ...
    @property
    def timestamp(self) -> int:
        """Timestamp of block (ms since Unix epoch)"""
        ...
    @property
    def n_bits(self) -> int:
        """Current difficulty encoded in n-bits form"""
        ...
    @property
    def height(self) -> int:
        """Block height"""
        ...
    @property
    def extension_root(self) -> int:
        """Root hash of extension section"""
        ...

@typing.final
class PreHeader:
    """
    Block header with the current `spendingTransaction`, that can be predicted
    """
    def __init__(self, header: Header):
        ...
    ...

@typing.final
class Parameters:
    """Parameters of blockchaian that can be adjusted by voting"""
    @classmethod
    def default(cls):
        """Return default parameters that were set at genesis. This is sufficient to use for non consensus-critical applications such as wallets"""
        ...
    @classmethod
    def from_json(cls, json: str | dict) -> Self:
        ...



@typing.final
class ErgoStateContext:
    """Blockchain state used for signing and verification"""
    def __init__(self, pre_header: PreHeader, headers: list[Header], parameters: Parameters):
        """:param pre_header: Predicted next-block header
        :param headers: Fixed number (10) of last block headers in descending order (first header is the newest one)
        :param parameters: Blockchain parameters
        :raises ValueError: if `len(headers)` != 10
        """
    ...
