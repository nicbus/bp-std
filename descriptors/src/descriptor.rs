// Modern, minimalistic & standard-compliant cold wallet library.
//
// SPDX-License-Identifier: Apache-2.0
//
// Written in 2020-2023 by
//     Dr Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2020-2023 LNP/BP Standards Association. All rights reserved.
// Copyright (C) 2020-2023 Dr Maxim Orlovsky. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::collections::BTreeSet;
use std::{iter, vec};

use derive::{
    CompressedPk, Derive, DeriveCompr, DeriveScripts, DeriveSet, DeriveXOnly, DerivedScript,
    KeyOrigin, Keychain, NormalIndex, Sats, TapDerivation, Terminal, XOnlyPk, XpubDerivable,
    XpubSpec,
};
use indexmap::IndexMap;

use crate::{TrKey, Wpkh};

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Display)]
#[display(lowercase)]
pub enum SpkClass {
    Bare,
    P2pkh,
    P2sh,
    P2wpkh,
    P2wsh,
    P2tr,
}

impl SpkClass {
    pub const fn dust_limit(self) -> Sats {
        match self {
            SpkClass::Bare => Sats(0),
            SpkClass::P2pkh => Sats(546),
            SpkClass::P2sh => Sats(540),
            SpkClass::P2wpkh => Sats(294),
            SpkClass::P2wsh | SpkClass::P2tr => Sats(330),
        }
    }
}

pub trait Descriptor<K = XpubDerivable, V = ()>: DeriveScripts {
    type KeyIter<'k>: Iterator<Item = &'k K>
    where
        Self: 'k,
        K: 'k;

    type VarIter<'v>: Iterator<Item = &'v V>
    where
        Self: 'v,
        V: 'v;

    type XpubIter<'x>: Iterator<Item = &'x XpubSpec>
    where Self: 'x;

    fn class(&self) -> SpkClass;

    fn keys(&self) -> Self::KeyIter<'_>;
    fn vars(&self) -> Self::VarIter<'_>;
    fn xpubs(&self) -> Self::XpubIter<'_>;

    fn compr_keyset(&self, terminal: Terminal) -> IndexMap<CompressedPk, KeyOrigin>;
    fn xonly_keyset(&self, terminal: Terminal) -> IndexMap<XOnlyPk, TapDerivation>;
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, From)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(
        crate = "serde_crate",
        rename_all = "camelCase",
        bound(
            serialize = "S::Compr: serde::Serialize, S::XOnly: serde::Serialize",
            deserialize = "S::Compr: serde::Deserialize<'de>, S::XOnly: serde::Deserialize<'de>"
        )
    )
)]
#[non_exhaustive]
pub enum StdDescr<S: DeriveSet = XpubDerivable> {
    /*
    #[from]
    Bare(Bare<S::Legacy>),

    #[from]
    Pkh(Pkh<S::Legacy>),

    #[from]
    ShMulti(ShMulti<S::Legacy>),

    #[from]
    ShSortedMulti(ShSortedMulti<S::Legacy>),

    #[from]
    ShTlMulti(ShTlMulti<S::Legacy>),

    #[from]
    ShTemplate(ShTemplate<S::Legacy>),
     */
    #[from]
    Wpkh(Wpkh<S::Compr>),

    /*
    #[from]
    WshMulti(WshMulti<S::Compr>),

    #[from]
    WshSortedMulti(WshSortedMulti<S::Compr>),

    #[from]
    WshTlMulti(WshTlMulti<S::Compr>),

    #[from]
    WshTemplate(ShTemplate<S::Compr>),
     */
    #[from]
    TrKey(TrKey<S::XOnly>),
    /*
    #[from]
    TrMusig(TrMusig<S::XOnly>),

    #[from]
    TrMulti(TrMulti<S::XOnly>),

    #[from]
    TrTlMulti(TrTlMulti<S::XOnly>),

    #[from]
    TrTree(TrTree<S::XOnly>),

    // This should go into LNP:
    Bolt(Bolt<S::Compr>)

    // The rest should go to RGB:
    #[from]
    TapretKey(TapretKey<S::XOnly),

    #[from]
    TapretMusig(TapretMusig<S::XOnly>),

    #[from]
    TrMulti(TapretMulti<S::XOnly>),

    #[from]
    TapretTlMulti(TapretTlMulti<S::XOnly>),

    #[from]
    TapretTree(TapretTree<S::XOnly>),
     */
}

impl<S: DeriveSet> Derive<DerivedScript> for StdDescr<S> {
    fn default_keychain(&self) -> Keychain {
        match self {
            StdDescr::Wpkh(d) => d.default_keychain(),
            StdDescr::TrKey(d) => d.default_keychain(),
        }
    }

    fn keychains(&self) -> BTreeSet<Keychain> {
        match self {
            StdDescr::Wpkh(d) => d.keychains(),
            StdDescr::TrKey(d) => d.keychains(),
        }
    }

    fn derive(
        &self,
        keychain: impl Into<Keychain>,
        index: impl Into<NormalIndex>,
    ) -> DerivedScript {
        match self {
            StdDescr::Wpkh(d) => d.derive(keychain, index),
            StdDescr::TrKey(d) => d.derive(keychain, index),
        }
    }
}

impl<K: DeriveSet<Compr = K, XOnly = K> + DeriveCompr + DeriveXOnly> Descriptor<K> for StdDescr<K>
where Self: Derive<DerivedScript>
{
    type KeyIter<'k> = vec::IntoIter<&'k K> where Self: 'k, K: 'k;
    type VarIter<'v> = iter::Empty<&'v ()> where Self: 'v, (): 'v;
    type XpubIter<'x> = vec::IntoIter<&'x XpubSpec> where Self: 'x;

    fn class(&self) -> SpkClass {
        match self {
            StdDescr::Wpkh(d) => d.class(),
            StdDescr::TrKey(d) => d.class(),
        }
    }

    fn keys(&self) -> Self::KeyIter<'_> {
        match self {
            StdDescr::Wpkh(d) => d.keys().collect::<Vec<_>>(),
            StdDescr::TrKey(d) => d.keys().collect::<Vec<_>>(),
        }
        .into_iter()
    }

    fn vars(&self) -> Self::VarIter<'_> { iter::empty() }

    fn xpubs(&self) -> Self::XpubIter<'_> {
        match self {
            StdDescr::Wpkh(d) => d.xpubs().collect::<Vec<_>>(),
            StdDescr::TrKey(d) => d.xpubs().collect::<Vec<_>>(),
        }
        .into_iter()
    }

    fn compr_keyset(&self, terminal: Terminal) -> IndexMap<CompressedPk, KeyOrigin> {
        match self {
            StdDescr::Wpkh(d) => d.compr_keyset(terminal),
            StdDescr::TrKey(d) => d.compr_keyset(terminal),
        }
    }

    fn xonly_keyset(&self, terminal: Terminal) -> IndexMap<XOnlyPk, TapDerivation> {
        match self {
            StdDescr::Wpkh(d) => d.xonly_keyset(terminal),
            StdDescr::TrKey(d) => d.xonly_keyset(terminal),
        }
    }
}
