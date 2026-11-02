use derive_more::{From, Into};
use ergo_lib::ergotree_ir::{
    bigint256::BigInt256,
    mir::{
        constant::{self, Literal, TryExtractInto},
        value::{CollKind, NativeColl},
    },
    serialization::SigmaSerializable,
    types::{stuple::STuple, stype},
    unsignedbigint256::UnsignedBigInt,
};
use pyo3::{
    exceptions::PyValueError,
    prelude::*,
    types::{PyBytes, PyDict, PyFloat, PyInt, PyList, PyTuple, PyType},
    IntoPyObjectExt,
};

use crate::{
    errors::{SigmaParsingError, SigmaSerializationError},
    to_value_error,
};

use super::{ec_point::EcPoint, ergo_box::ErgoBox};

#[pyclass]
#[allow(clippy::enum_variant_names)]
pub(crate) enum SType {
    SUnit(),
    SBoolean(),
    SByte(),
    SShort(),
    SInt(),
    SLong(),
    SBigInt(),
    SUnsignedBigInt(),
    SGroupElement(),
    SSigmaProp(),
    SBox(),
    SAvlTree(),
    SOption(Py<SType>),
    SColl(Py<SType>),
    STuple(Py<PyTuple>),
    SString(),
    SHeader(),
}

impl SType {
    fn from_stype(py: Python, tpe: &stype::SType) -> PyResult<Self> {
        Ok(match tpe {
            stype::SType::SUnit => SType::SUnit(),
            stype::SType::SBoolean => SType::SBoolean(),
            stype::SType::SByte => SType::SByte(),
            stype::SType::SShort => SType::SShort(),
            stype::SType::SInt => SType::SInt(),
            stype::SType::SLong => SType::SLong(),
            stype::SType::SBigInt => SType::SBigInt(),
            stype::SType::SUnsignedBigInt => SType::SUnsignedBigInt(),
            stype::SType::SGroupElement => SType::SGroupElement(),
            stype::SType::SSigmaProp => SType::SSigmaProp(),
            stype::SType::SBox => SType::SBox(),
            stype::SType::SAvlTree => SType::SAvlTree(),
            stype::SType::SOption(stype) => {
                SType::SOption(Py::new(py, SType::from_stype(py, stype)?)?)
            }
            stype::SType::SColl(elem_tpe) => {
                SType::SColl(SType::from_stype(py, elem_tpe)?.into_pyobject(py)?.unbind())
            }
            stype::SType::STuple(stuple) => {
                let elements = stuple
                    .items
                    .iter()
                    .map(|tpe| SType::from_stype(py, tpe))
                    .collect::<PyResult<Vec<_>>>()?;
                SType::STuple(PyTuple::new(py, elements)?.unbind())
            }
            stype::SType::SString => SType::SString(),
            stype::SType::SHeader => SType::SHeader(),
            // TODO: consider adding SFunc type to python after 6.0 since it might be useful then
            stype::SType::STypeVar(_)
            | stype::SType::SAny
            | stype::SType::SPreHeader
            | stype::SType::SGlobal
            | stype::SType::SContext
            | stype::SType::SFunc(_) => return Err(PyValueError::new_err("unexpected tpe")),
        })
    }
    fn to_stype(&self, py: Python) -> PyResult<stype::SType> {
        Ok(match self {
            SType::SUnit() => stype::SType::SUnit,
            SType::SBoolean() => stype::SType::SBoolean,
            SType::SByte() => stype::SType::SByte,
            SType::SShort() => stype::SType::SShort,
            SType::SInt() => stype::SType::SInt,
            SType::SLong() => stype::SType::SLong,
            SType::SBigInt() => stype::SType::SBigInt,
            SType::SUnsignedBigInt() => stype::SType::SUnsignedBigInt,
            SType::SGroupElement() => stype::SType::SGroupElement,
            SType::SSigmaProp() => stype::SType::SSigmaProp,
            SType::SBox() => stype::SType::SBox,
            SType::SAvlTree() => stype::SType::SAvlTree,
            SType::SOption(inner_tpe) => {
                stype::SType::SOption(inner_tpe.borrow(py).to_stype(py)?.into())
            }
            SType::SColl(elem_tpe) => stype::SType::SColl(elem_tpe.borrow(py).to_stype(py)?.into()),
            SType::STuple(tpes) => stype::SType::STuple(
                STuple::try_from(
                    tpes.bind(py)
                        .iter()
                        .map(|tpe| -> PyResult<stype::SType> {
                            tpe.cast::<SType>()?.get().to_stype(py)
                        })
                        .collect::<PyResult<Vec<_>>>()?,
                )
                .map_err(to_value_error)?,
            ),
            SType::SString() => stype::SType::SString,
            SType::SHeader() => stype::SType::SHeader,
        })
    }
}

#[pymethods]
impl SType {
    fn __eq__(&self, other: &SType, py: Python) -> PyResult<bool> {
        Ok(match (self, other) {
            (SType::SUnit(), SType::SUnit()) => true,
            (SType::SBoolean(), SType::SBoolean()) => true,
            (SType::SByte(), SType::SByte()) => true,
            (SType::SShort(), SType::SShort()) => true,
            (SType::SInt(), SType::SInt()) => true,
            (SType::SLong(), SType::SLong()) => true,
            (SType::SBigInt(), SType::SBigInt()) => true,
            (SType::SSigmaProp(), SType::SSigmaProp()) => true,
            (SType::SBox(), SType::SBox()) => true,
            (SType::SAvlTree(), SType::SAvlTree()) => true,
            (SType::SOption(tpe1), SType::SOption(tpe2)) => {
                tpe1.borrow(py).__eq__(&tpe2.borrow(py), py)?
            }
            (SType::SColl(tpe1), SType::SColl(tpe2)) => {
                tpe1.borrow(py).__eq__(&tpe2.borrow(py), py)?
            }
            (SType::STuple(t1), SType::STuple(t2)) => t1
                .bind(py)
                .iter()
                .zip(t2.bind(py).iter())
                .map(|(t1, t2)| -> PyResult<bool> {
                    t1.cast_into::<SType>()?
                        .get()
                        .__eq__(t2.cast_into::<SType>()?.get(), py)
                })
                .reduce(|res1, res2| res1.and_then(|res1| res2.map(|res2| res1 == res2)))
                .transpose()?
                .unwrap_or(true),
            (SType::SString(), SType::SString()) => true,
            (SType::SHeader(), SType::SHeader()) => true,
            _ => false,
        })
    }
}
/// Constant value that can be used in ErgoBox registers, ErgoTree constants and ContextExtension
#[pyclass(eq, from_py_object)]
#[derive(PartialEq, Eq, Clone, Debug, From, Into)]
pub(crate) struct Constant(constant::Constant);

#[pymethods]
impl Constant {
    #[new]
    #[pyo3(signature = (arg, elem_tpe = None))]
    fn new(arg: &Bound<'_, PyAny>, elem_tpe: Option<&SType>, py: Python) -> PyResult<Self> {
        if arg.is_exact_instance_of::<PyInt>() | arg.is_exact_instance_of::<PyFloat>() {
            return Err(PyValueError::new_err("Constant.new does not support numeric type as argument. Use Constant.from_i64, from_i32, etc instead"));
        }
        if let Ok(bytes) = arg.extract::<&[u8]>() {
            return Ok(Self(constant::Constant::from(bytes.to_owned())));
        }
        if let Ok(tuple) = arg.cast_exact::<PyTuple>() {
            return from_tuple(tuple);
        }
        if let Ok(arr) = arg.extract::<Vec<Constant>>() {
            return Ok(Self(
                constant::Constant::coll_from_iter(
                    arr.into_iter().map(|constant| constant.0),
                    elem_tpe.map(|tpe| tpe.to_stype(py)).transpose()?,
                )
                .map_err(to_value_error)?,
            ));
        }
        if let Ok(ec_point) = arg.extract::<EcPoint>() {
            return Ok(Self(ec_point.0.into()));
        }
        if let Ok(bool) = arg.extract::<bool>() {
            return Ok(Self(bool.into()));
        }
        if let Ok(ergo_box) = arg.extract::<ErgoBox>() {
            return Ok(Self(ergo_box.0.into()));
        }
        // TODO (6.0): Add Header, Opt, UnsignedBigInt
        Err(PyValueError::new_err("unexpected value"))
    }

    #[getter]
    fn tpe(&self, py: Python) -> PyResult<SType> {
        SType::from_stype(py, &self.0.tpe)
    }
    #[getter]
    fn value(&self, py: Python) -> PyResult<Py<PyAny>> {
        constant_to_py(py, self.clone())
    }

    #[classmethod]
    fn from_u256(_: &Bound<'_, PyType>, py: Python, v: &Bound<'_, PyInt>) -> PyResult<Self> {
        let kwargs = PyDict::new(py);
        kwargs.set_item("length", 32)?;
        kwargs.set_item("byteorder", "big")?;
        kwargs.set_item("signed", false)?;
        let be_bytes: [u8; 32] = v.call_method("to_bytes", (), Some(&kwargs))?.extract()?;
        Ok(Self(constant::Constant::from(
            #[allow(clippy::unwrap_used)] // length is guaranted to be 32 bytes
            UnsignedBigInt::from_be_slice(&be_bytes[..]).unwrap(),
        )))
    }

    #[classmethod]
    fn from_i256(_: &Bound<'_, PyType>, py: Python, v: &Bound<'_, PyInt>) -> PyResult<Self> {
        let kwargs = PyDict::new(py);
        kwargs.set_item("length", 32)?;
        kwargs.set_item("byteorder", "big")?;
        kwargs.set_item("signed", true)?;
        let be_bytes: [u8; 32] = v.call_method("to_bytes", (), Some(&kwargs))?.extract()?;
        Ok(Self(constant::Constant::from(
            #[allow(clippy::unwrap_used)] // length is guaranted to be 32 bytes
            BigInt256::from_be_slice(&be_bytes[..]).unwrap(),
        )))
    }

    #[classmethod]
    fn from_i64(_: &Bound<'_, PyType>, v: i64) -> Constant {
        Constant(constant::Constant::from(v))
    }

    #[classmethod]
    fn from_i32(_: &Bound<'_, PyType>, v: i32) -> Constant {
        Constant(constant::Constant::from(v))
    }

    #[classmethod]
    fn from_i16(_: &Bound<'_, PyType>, v: i16) -> Constant {
        Constant(constant::Constant::from(v))
    }

    #[classmethod]
    fn from_i8(_: &Bound<'_, PyType>, v: i8) -> Constant {
        Constant(constant::Constant::from(v))
    }

    /// Serialize Constant as byte array
    fn __bytes__(&self) -> PyResult<Vec<u8>> {
        self.0
            .sigma_serialize_bytes()
            .map_err(SigmaSerializationError::from)
            .map_err(Into::into)
    }

    /// Parse serialized Constant from byte-array
    #[classmethod]
    fn from_bytes(_: &Bound<'_, PyType>, b: &[u8]) -> PyResult<Self> {
        constant::Constant::sigma_parse_bytes(b)
            .map(Self)
            .map_err(SigmaParsingError::from)
            .map_err(Into::into)
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }
}

#[allow(clippy::unwrap_used)]
#[allow(clippy::unreachable)]
#[allow(clippy::wildcard_enum_match_arm)]
fn constant_to_py(py: Python, c: Constant) -> PyResult<Py<PyAny>> {
    Ok(match c.0.tpe {
        stype::SType::SBoolean => c.0.try_extract_into::<bool>().unwrap().into_py_any(py)?,
        stype::SType::SShort => c.0.try_extract_into::<i16>().unwrap().into_py_any(py)?,
        stype::SType::SByte => c.0.try_extract_into::<i8>().unwrap().into_py_any(py)?,
        stype::SType::SInt => c.0.try_extract_into::<i32>().unwrap().into_py_any(py)?,
        stype::SType::SLong => c.0.try_extract_into::<i64>().unwrap().into_py_any(py)?,
        stype::SType::SBigInt => {
            ergo_bigint_to_python(py, c.0.v.try_extract_into::<BigInt256>().unwrap())?
        }
        stype::SType::SUnsignedBigInt => {
            ergo_unsigned_bigint_to_python(py, c.0.v.try_extract_into::<UnsignedBigInt>().unwrap())?
        }
        stype::SType::SColl(_) => match c.0.v {
            Literal::Coll(CollKind::NativeColl(NativeColl::CollByte(v))) => {
                PyBytes::new(py, &v.iter().map(|&i| i as u8).collect::<Vec<_>>()).into_py_any(py)?
            }
            Literal::Coll(CollKind::WrappedColl { elem_tpe, items }) => {
                let arr = PyList::new(
                    py,
                    items.iter().map(|item| {
                        Constant(constant::Constant {
                            tpe: elem_tpe.clone(),
                            v: item.clone(),
                        })
                    }),
                )?;
                arr.into_py_any(py)?
            }
            _ => unreachable!(),
        },
        stype::SType::STuple(ref item_tpes) => {
            let tuple = match c.0.v {
                Literal::Tup(v) => PyTuple::new(
                    py,
                    v.into_iter()
                        .zip(item_tpes.clone().items.into_iter())
                        .map(|(v, tpe)| Constant(constant::Constant { tpe, v })),
                )?,
                _ => unreachable!(),
            };
            tuple.into_py_any(py)?
        }
        stype::SType::SGroupElement => {
            EcPoint(c.0.v.try_extract_into().unwrap()).into_py_any(py)?
        }
        stype::SType::SBox => ErgoBox(c.0.v.try_extract_into().unwrap()).into_py_any(py)?,
        stype::SType::SAny
        | stype::SType::SUnit
        | stype::SType::STypeVar(_)
        | stype::SType::SSigmaProp
        | stype::SType::SAvlTree
        | stype::SType::SOption(_)
        | stype::SType::SFunc(_)
        | stype::SType::SContext
        | stype::SType::SString
        | stype::SType::SHeader
        | stype::SType::SPreHeader
        | stype::SType::SGlobal => return Err(PyValueError::new_err("unsupported constant tpe")),
    })
}

fn ergo_bigint_to_python(py: Python, bigint: BigInt256) -> PyResult<Py<PyAny>> {
    let be_bytes: [u8; 32] = bigint.to_be_bytes();
    let kwargs = PyDict::new(py);
    kwargs.set_item("bytes", be_bytes)?;
    kwargs.set_item("byteorder", "big")?;
    kwargs.set_item("signed", true)?;
    let py_int = py.get_type::<PyInt>();
    let bound = py_int.call_method("from_bytes", (), Some(&kwargs))?;
    Ok(bound.unbind())
}

fn ergo_unsigned_bigint_to_python(py: Python, bigint: UnsignedBigInt) -> PyResult<Py<PyAny>> {
    let be_bytes: [u8; 32] = bigint.to_be_bytes();
    let kwargs = PyDict::new(py);
    kwargs.set_item("bytes", be_bytes)?;
    kwargs.set_item("byteorder", "big")?;
    kwargs.set_item("signed", false)?;
    let py_int = py.get_type::<PyInt>();
    let be_bytes: [u8; 32] = bigint.to_be_bytes();
    let bound = py_int.call_method("from_bytes", (), Some(&kwargs))?;
    Ok(bound.unbind())
}

fn from_tuple(tuple: &Bound<'_, PyTuple>) -> PyResult<Constant> {
    let mut tpes = vec![];
    let mut items = vec![];
    for item in tuple.iter() {
        let Constant(constant::Constant { tpe, v }) = item.extract::<Constant>()?;
        tpes.push(tpe);
        items.push(v);
    }
    Ok(Constant(constant::Constant {
        tpe: STuple::try_from(tpes).map_err(to_value_error)?.into(),
        v: Literal::Tup(items.try_into().map_err(to_value_error)?),
    }))
}
