// Copyright (c) 2019-2022 Parity Technologies Limited
// This file is part of subxt.
//
// Permission is hereby granted, free of charge, to any
// person obtaining a copy of this software and associated
// documentation files (the "Software"), to deal in the
// Software without restriction, including without
// limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software
// is furnished to do so, subject to the following
// conditions:
//
// The above copyright notice and this permission notice
// shall be included in all copies or substantial portions
// of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
// ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
// TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
// PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
// SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
// CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
// IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

use crate::utils::{
    dispatch_error::{
        ArrayDispatchError,
        LegacyDispatchError,
        NamedFieldDispatchError,
    },
    generate_metadata_from_pallets_custom_dispatch_error,
};
use frame_metadata::RuntimeMetadataPrefixed;

pub fn metadata_array_dispatch_error() -> RuntimeMetadataPrefixed {
    generate_metadata_from_pallets_custom_dispatch_error::<ArrayDispatchError>(vec![])
}

pub fn metadata_legacy_dispatch_error() -> RuntimeMetadataPrefixed {
    generate_metadata_from_pallets_custom_dispatch_error::<LegacyDispatchError>(vec![])
}

pub fn metadata_named_field_dispatch_error() -> RuntimeMetadataPrefixed {
    generate_metadata_from_pallets_custom_dispatch_error::<NamedFieldDispatchError>(
        vec![],
    )
}