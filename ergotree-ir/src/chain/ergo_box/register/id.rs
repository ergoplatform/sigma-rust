use std::fmt::Display;

use derive_more::From;
use thiserror::Error;

/// Box register id (0-9)
#[derive(PartialEq, Eq, Debug, Clone, Copy, From)]
pub enum RegisterId {
    /// Id for mandatory registers (0-3)
    MandatoryRegisterId(MandatoryRegisterId),
    /// Id for non-mandotory registers (4-9)
    NonMandatoryRegisterId(NonMandatoryRegisterId),
}

impl RegisterId {
    /// R0 register id (box.value)
    pub const R0: RegisterId = RegisterId::MandatoryRegisterId(MandatoryRegisterId::R0);
    /// R1 register id (serialized ErgoTree)
    pub const R1: RegisterId = RegisterId::MandatoryRegisterId(MandatoryRegisterId::R1);
    /// R2 register id (tokens)
    pub const R2: RegisterId = RegisterId::MandatoryRegisterId(MandatoryRegisterId::R2);
    /// R2 register id (creationInfo)
    pub const R3: RegisterId = RegisterId::MandatoryRegisterId(MandatoryRegisterId::R3);
}

/// Register id out of bounds error (not in 0-9 range)
#[derive(Error, PartialEq, Eq, Debug, Clone)]
#[error("register id {0} is out of bounds (0 - 9)")]
pub struct RegisterIdOutOfBounds(pub i8);

impl TryFrom<i8> for RegisterId {
    type Error = RegisterIdOutOfBounds;

    fn try_from(value: i8) -> Result<Self, Self::Error> {
        if value < 0 {
            return Err(RegisterIdOutOfBounds(value));
        }
        let v = value as usize;
        if v < NonMandatoryRegisterId::START_INDEX {
            Ok(RegisterId::MandatoryRegisterId(value.try_into()?))
        } else if v <= NonMandatoryRegisterId::END_INDEX {
            Ok(RegisterId::NonMandatoryRegisterId(value.try_into()?))
        } else {
            Err(RegisterIdOutOfBounds(value))
        }
    }
}

impl TryFrom<u8> for RegisterId {
    type Error = RegisterIdOutOfBounds;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        RegisterId::try_from(value as i8)
    }
}

impl Display for RegisterId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegisterId::MandatoryRegisterId(id) => write!(f, "{}", id),
            RegisterId::NonMandatoryRegisterId(id) => write!(f, "{}", id),
        }
    }
}

/// Register ids that every box have (box properties exposed as registers)
#[derive(PartialEq, Eq, Debug, Clone, Copy, derive_more::Display)]
pub enum MandatoryRegisterId {
    /// Monetary value, in Ergo tokens
    R0 = 0,
    /// Guarding script
    R1 = 1,
    /// Secondary tokens
    R2 = 2,
    /// Reference to transaction and output id where the box was created
    R3 = 3,
}

impl TryFrom<i8> for MandatoryRegisterId {
    type Error = RegisterIdOutOfBounds;

    fn try_from(value: i8) -> Result<Self, Self::Error> {
        match value {
            v if v == MandatoryRegisterId::R0 as i8 => Ok(MandatoryRegisterId::R0),
            v if v == MandatoryRegisterId::R1 as i8 => Ok(MandatoryRegisterId::R1),
            v if v == MandatoryRegisterId::R2 as i8 => Ok(MandatoryRegisterId::R2),
            v if v == MandatoryRegisterId::R3 as i8 => Ok(MandatoryRegisterId::R3),
            _ => Err(RegisterIdOutOfBounds(value)),
        }
    }
}

/// newtype for additional registers R4 - R9
#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy, derive_more::Display)]
#[cfg_attr(feature = "json", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "json", serde(into = "String", try_from = "String"))]
#[repr(u8)]
pub enum NonMandatoryRegisterId {
    /// id for R4 register
    R4 = 4,
    /// id for R5 register
    R5 = 5,
    /// id for R6 register
    R6 = 6,
    /// id for R7 register
    R7 = 7,
    /// id for R8 register
    R8 = 8,
    /// id for R9 register
    R9 = 9,
}

impl NonMandatoryRegisterId {
    /// starting index for non-mandatory registers
    pub const START_INDEX: usize = 4;
    /// end index for non-mandatory registers
    pub const END_INDEX: usize = 9;

    /// max number of registers
    pub const NUM_REGS: usize = 6;

    /// all register ids
    pub const REG_IDS: [NonMandatoryRegisterId; NonMandatoryRegisterId::NUM_REGS] = [
        NonMandatoryRegisterId::R4,
        NonMandatoryRegisterId::R5,
        NonMandatoryRegisterId::R6,
        NonMandatoryRegisterId::R7,
        NonMandatoryRegisterId::R8,
        NonMandatoryRegisterId::R9,
    ];

    /// get register by it's index starting from 0
    /// `i` is expected to be in range 0..[`Self::NUM_REGS`] , otherwise panic
    pub fn get_by_zero_index(i: usize) -> NonMandatoryRegisterId {
        assert!(i < NonMandatoryRegisterId::NUM_REGS);
        NonMandatoryRegisterId::REG_IDS[i]
    }
}

impl From<NonMandatoryRegisterId> for String {
    fn from(v: NonMandatoryRegisterId) -> Self {
        format!("R{}", v as u8)
    }
}

impl TryFrom<String> for NonMandatoryRegisterId {
    type Error = NonMandatoryRegisterIdParsingError;
    fn try_from(str: String) -> Result<Self, Self::Error> {
        if str.len() == 2 && &str[..1] == "R" {
            let index = str[1..2]
                .parse::<usize>()
                .map_err(|_| NonMandatoryRegisterIdParsingError())?;
            if (NonMandatoryRegisterId::START_INDEX..=NonMandatoryRegisterId::END_INDEX)
                .contains(&index)
            {
                Ok(NonMandatoryRegisterId::get_by_zero_index(
                    index - NonMandatoryRegisterId::START_INDEX,
                ))
            } else {
                Err(NonMandatoryRegisterIdParsingError())
            }
        } else {
            Err(NonMandatoryRegisterIdParsingError())
        }
    }
}

impl TryFrom<i8> for NonMandatoryRegisterId {
    type Error = RegisterIdOutOfBounds;

    fn try_from(value: i8) -> Result<Self, Self::Error> {
        let v_usize = value as usize;
        if (NonMandatoryRegisterId::START_INDEX..=NonMandatoryRegisterId::END_INDEX)
            .contains(&v_usize)
        {
            Ok(NonMandatoryRegisterId::get_by_zero_index(
                v_usize - NonMandatoryRegisterId::START_INDEX,
            ))
        } else {
            Err(RegisterIdOutOfBounds(value))
        }
    }
}

#[derive(Error, PartialEq, Eq, Debug, Clone)]
#[error("failed to parse register id")]
/// Error for failed parsing of the register id from string
pub struct NonMandatoryRegisterIdParsingError();
