use std::collections::HashMap;

use neon::prelude::*;

pub trait ResultExtJs<T, E> {
    fn or_throw<'a>(self, cx: &mut impl Context<'a>) -> NeonResult<T>
        where E: std::fmt::Display;
}
impl<T, E> ResultExtJs<T, E> for Result<T, E> {
    fn or_throw<'a>(self, cx: &mut impl Context<'a>) -> NeonResult<T> 
        where E: std::fmt::Display
    {
        self.or_else(|e| cx.throw_error(e.to_string()))
    }
}

pub trait IntoJsValue {
    type Value: Value;
    fn into_js<'a>(self, cx: &mut impl Context<'a>) -> Handle<'a, Self::Value>;
}
pub trait TryIntoJsValue {
    type Value: Value;

    fn try_into_js<'a>(self, cx: &mut impl Context<'a>) -> JsResult<'a, Self::Value>;
}
impl<T: IntoJsValue> TryIntoJsValue for T {
    type Value = T::Value;

    fn try_into_js<'a>(self, cx: &mut impl Context<'a>) -> JsResult<'a, Self::Value> {
        Ok(self.into_js(cx))
    }
}

pub fn try_array_from_iter<'a, I>(iter: I, cx: &mut impl Context<'a>) -> JsResult<'a, JsArray> 
    where I: IntoIterator<Item: TryIntoJsValue>
{
    let array = cx.empty_array();
    for (i, el) in iter.into_iter().enumerate() {
        let n = cx.number(i as f64);
        let el = el.try_into_js(cx)?;
        array.set(cx, n, el)?;
    }

    Ok(array)
}
macro_rules! number {
    ($($t:ty),*) => {
        $(
            impl IntoJsValue for $t {
                type Value = JsNumber;
                fn into_js<'a>(self, cx: &mut impl Context<'a>) -> Handle<'a, Self::Value> {
                    cx.number(self)
                }
            }
        )*
    }
}

number! { u8, u16, u32, i8, i16, i32, f32, f64 }
impl IntoJsValue for &'_ str {
    type Value = JsString;

    fn into_js<'a>(self, cx: &mut impl Context<'a>) -> Handle<'a, Self::Value> {
        cx.string(self)
    }
}
impl IntoJsValue for bool {
    type Value = JsBoolean;
    
    fn into_js<'a>(self, cx: &mut impl Context<'a>) -> Handle<'a, Self::Value> {
        cx.boolean(self)
    }
    
}
impl IntoJsValue for () {
    type Value = JsUndefined;

    fn into_js<'a>(self, cx: &mut impl Context<'a>) -> Handle<'a, Self::Value> {
        cx.undefined()
    }
}

macro_rules! try_number {
    ($($t:ty),*) => {
        $(
            impl TryIntoJsValue for $t {
                type Value = JsNumber;
                fn try_into_js<'a>(self, cx: &mut impl Context<'a>) -> JsResult<'a, Self::Value> {
                    u32::try_from(self)
                        .map(|n| cx.number(n))
                        .or_throw(cx)
                }
            }
        )*
    }
}
try_number! { usize, isize, u64, i64, u128, i128 }

impl<T: TryIntoJsValue> TryIntoJsValue for Option<T> {
    type Value = JsValue;
    
    fn try_into_js<'a>(self, cx: &mut impl Context<'a>) -> JsResult<'a, Self::Value> {
        match self {
            Some(t) => t.try_into_js(cx).map(|e| e.upcast()),
            None => Ok(cx.undefined().upcast()),
        }
    }
}
impl<T: TryIntoJsValue> TryIntoJsValue for NeonResult<T> {
    type Value = T::Value;

    fn try_into_js<'a>(self, cx: &mut impl Context<'a>) -> JsResult<'a, Self::Value> {
        self?.try_into_js(cx)
    }
}
impl<T: TryIntoJsValue> TryIntoJsValue for Vec<T> {
    type Value = JsArray;

    fn try_into_js<'a>(self, cx: &mut impl Context<'a>) -> JsResult<'a, Self::Value> {
        try_array_from_iter(self, cx)
    }
}
impl<T: TryIntoJsValue> TryIntoJsValue for Box<[T]> {
    type Value = JsArray;

    fn try_into_js<'a>(self, cx: &mut impl Context<'a>) -> JsResult<'a, Self::Value> {
        try_array_from_iter(self, cx)
    }
}
impl<T: TryIntoJsValue, const N: usize> TryIntoJsValue for [T; N] {
    type Value = JsArray;

    fn try_into_js<'a>(self, cx: &mut impl Context<'a>) -> JsResult<'a, Self::Value> {
        try_array_from_iter(self, cx)
    }
}

impl<K: TryIntoJsValue, V: TryIntoJsValue> TryIntoJsValue for HashMap<K, V> {
    type Value = JsObject;

    fn try_into_js<'a>(self, cx: &mut impl Context<'a>) -> JsResult<'a, Self::Value> {
        let obj = cx.empty_object();
        for (k, v) in self.into_iter() {
            let k = k.try_into_js(cx)?;
            let v = v.try_into_js(cx)?;
            obj.set(cx, k, v)?;
        }

        Ok(obj)
    }
}