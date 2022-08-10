#[cfg(any(doc, feature = "simd"))]
use std::simd::{LaneCount, SupportedLaneCount};

use crate::color::Argb;
use crate::color::serial::StackBlurrableU32;
#[cfg(any(doc, feature = "simd"))]
use crate::color::simd::StackBlurrableU32xN;

mod __sealed {
	#[cfg(any(doc, feature = "simd"))]
	use std::simd::{LaneCount, SupportedLaneCount};

	use crate::StackBlurrable;

	pub trait Sealed {
		type B: StackBlurrable;
	}

	#[cfg(any(doc, feature = "simd"))]
	pub trait SealedSimd<const LANES: usize> where LaneCount<LANES>: SupportedLaneCount {
		type Bsimd: StackBlurrable;
	}

	#[cfg(any(doc, feature = "blend-srgb"))]
	pub trait SealedSrgb {
		type B: StackBlurrable;
	}

	#[cfg(any(doc, all(feature = "blend-srgb", feature = "simd")))]
	pub trait SealedSrgbSimd<const LANES: usize> where LaneCount<LANES>: SupportedLaneCount {
		type Bsimd: StackBlurrable;
	}
}

pub trait AsStackBlurrable: __sealed::Sealed + Sized {
	fn as_stackblurrable(&self) -> Self::B;
	fn from_stackblurrable(elem: Self::B) -> Self;
}

pub trait AsStackBlurrableSimd<const LANES: usize>: __sealed::SealedSimd<LANES> + AsStackBlurrable where LaneCount<LANES>: SupportedLaneCount {
	fn as_stackblurrable_simd(selves: [&Self; LANES]) -> Self::Bsimd;
	fn from_stackblurrable_simd(elems: Self::Bsimd) -> [Self; LANES];
}

#[cfg(any(doc, feature = "blend-srgb"))]
pub trait AsStackBlurrableSrgb: __sealed::SealedSrgb + Sized {
	fn as_stackblurrable_srgb(&self) -> Self::B;
	fn from_stackblurrable_srgb(elem: Self::B) -> Self;
}

#[cfg(any(doc, all(feature = "blend-srgb", feature = "simd")))]
pub trait AsStackBlurrableSrgbSimd<const LANES: usize>: __sealed::SealedSrgbSimd<LANES> + AsStackBlurrableSrgb where LaneCount<LANES>: SupportedLaneCount {
	fn as_stackblurrable_srgb_simd(selves: [&Self; LANES]) -> Self::Bsimd;
	fn from_stackblurrable_srgb_simd(elems: Self::Bsimd) -> [Self; LANES];
}

impl __sealed::Sealed for u32 {
	type B = Argb<StackBlurrableU32, 4>;
}

impl<const LANES: usize> __sealed::SealedSimd<LANES> for u32 where LaneCount<LANES>: SupportedLaneCount {
	type Bsimd = Argb<StackBlurrableU32xN<LANES>, 4>;
}

#[cfg(any(doc, feature = "blend-srgb"))]
impl __sealed::SealedSrgb for u32 {
	type B = Argb<StackBlurrableU32, 4>;
}

#[cfg(any(doc, all(feature = "blend-srgb", feature = "simd")))]
impl<const LANES: usize> __sealed::SealedSrgbSimd<LANES> for u32 where LaneCount<LANES>: SupportedLaneCount {
	type Bsimd = Argb<StackBlurrableU32xN<LANES>, 4>;
}

impl AsStackBlurrable for u32 {
	fn as_stackblurrable(&self) -> Self::B {
		Argb::from_u32(*self)
	}

	fn from_stackblurrable(elem: Self::B) -> Self {
		elem.to_u32()
	}
}

#[cfg(any(doc, feature = "simd"))]
impl<const LANES: usize> AsStackBlurrableSimd<LANES> for u32 where LaneCount<LANES>: SupportedLaneCount {
	fn as_stackblurrable_simd(selves: [&Self; LANES]) -> Self::Bsimd {
		Argb::<StackBlurrableU32xN<LANES>, 4>::from_u32xN(selves.map(|i| *i))
	}

	fn from_stackblurrable_simd(elems: Self::Bsimd) -> [Self; LANES] {
		elems.to_u32xN()
	}
}

#[cfg(any(doc, feature = "blend-srgb"))]
impl AsStackBlurrableSrgb for u32 {
	fn as_stackblurrable_srgb(&self) -> Self::B {
		Argb::from_u32_srgb(*self)
	}

	fn from_stackblurrable_srgb(elem: Self::B) -> Self {
		elem.to_u32_srgb()
	}
}

#[cfg(any(doc, all(feature = "blend-srgb", feature = "simd")))]
impl<const LANES: usize> AsStackBlurrableSrgbSimd<LANES> for u32 where LaneCount<LANES>: SupportedLaneCount {
	fn as_stackblurrable_srgb_simd(selves: [&Self; LANES]) -> Self::Bsimd {
		Argb::<StackBlurrableU32xN<LANES>, 4>::from_u32xN_srgb(selves.map(|i| *i))
	}

	fn from_stackblurrable_srgb_simd(elems: Self::Bsimd) -> [Self; LANES] {
		elems.to_u32xN_srgb()
	}
}

macro_rules! rgb {
	($rgb:ty[$components:tt] { $($n:ident),+ } as $ty:ty) => {
#[cfg(feature = "rgb")]
impl __sealed::Sealed for $rgb {
	type B = Argb<StackBlurrableU32, $components>;
}

#[cfg(all(feature = "rgb", feature = "simd"))]
impl<const LANES: usize> __sealed::SealedSimd<LANES> for $rgb where LaneCount<LANES>: SupportedLaneCount {
	type Bsimd = Argb<StackBlurrableU32xN<LANES>, $components>;
}

#[cfg(feature = "rgb")]
impl AsStackBlurrable for $rgb {
	fn as_stackblurrable(&self) -> Self::B {
		Argb([
			$(StackBlurrableU32(self.$n as u32)),+
		])
	}

	fn from_stackblurrable(elem: Self::B) -> Self {
		let [$($n),+] = elem.0;

		Self {
			$($n: $n.0 as $ty),+
		}
	}
}

#[cfg(all(feature = "rgb", feature = "simd"))]
impl<const LANES: usize> AsStackBlurrableSimd<LANES> for $rgb where LaneCount<LANES>: SupportedLaneCount {
	fn as_stackblurrable_simd(selves: [&Self; LANES]) -> Self::Bsimd {
		$(let $n = std::simd::Simd::<u32, LANES>::from_array(selves.map(|rgb| rgb.$n as u32));)+
		Argb([$(StackBlurrableU32xN::<LANES>($n)),+])
	}

	fn from_stackblurrable_simd(elem: Self::Bsimd) -> [Self; LANES] {
		let [$($n),+] = elem.0.map(|i| i.0.to_array());

		let mut countup = 0usize..;
		[(); LANES].map(move |_| {
			let i = countup.next().unwrap();
			Self { $($n: $n[i] as $ty),+ }
		})
	}
}
	}
}

macro_rules! gray {
	($gray:ty [$components:tt]($($idx:tt),+) as $ty:ty) => {
#[cfg(feature = "rgb")]
impl __sealed::Sealed for $gray {
	type B = Argb<StackBlurrableU32, $components>;
}

#[cfg(all(feature = "rgb", feature = "simd"))]
impl<const LANES: usize> __sealed::SealedSimd<LANES> for $gray where LaneCount<LANES>: SupportedLaneCount {
	type Bsimd = Argb<StackBlurrableU32xN<LANES>, $components>;
}

#[cfg(feature = "rgb")]
impl AsStackBlurrable for $gray {
	fn as_stackblurrable(&self) -> Self::B {
		Argb([$(StackBlurrableU32(self.$idx as u32)),+])
	}

	fn from_stackblurrable(elem: Self::B) -> Self {
		Self($(elem.0[$idx].0 as $ty),+)
	}
}

#[cfg(all(feature = "rgb", feature = "simd"))]
impl<const LANES: usize> AsStackBlurrableSimd<LANES> for $gray where LaneCount<LANES>: SupportedLaneCount {
	fn as_stackblurrable_simd(selves: [&Self; LANES]) -> Self::Bsimd {
		Argb([$(StackBlurrableU32xN(std::simd::Simd::from_array(selves.map(|gray| gray.$idx as u32)))),+])
	}

	fn from_stackblurrable_simd(elem: Self::Bsimd) -> [Self; LANES] {
		let arr = elem.0.map(|i| i.0.to_array());

		let mut countup = 0usize..;
		[(); LANES].map(move |_| {
			let i = countup.next().unwrap();
			Self($(arr[$idx][i] as $ty),+)
		})
	}
}
	}
}

rgb!(rgb::RGB8[3] { r, g, b } as u8);
rgb!(rgb::RGB16[3] { r, g, b } as u16);

rgb!(rgb::RGBA8[4] { r, g, b, a } as u8);
rgb!(rgb::RGBA16[4] { r, g, b, a } as u16);

rgb!(rgb::alt::BGR8[3] { b, g, r } as u8);
rgb!(rgb::alt::BGR16[3] { b, g, r } as u16);

rgb!(rgb::alt::BGRA8[4] { b, g, r, a } as u8);
rgb!(rgb::alt::BGRA16[4] { b, g, r, a } as u16);

#[cfg(feature = "rgb_argb")]
rgb!(rgb::alt::ARGB8[4] { r, g, b, a } as u8);
#[cfg(feature = "rgb_argb")]
rgb!(rgb::alt::ARGB16[4] { r, g, b, a } as u16);

gray!(rgb::alt::GRAY8[1](0) as u8);
gray!(rgb::alt::GRAY16[1](0) as u16);

gray!(rgb::alt::GRAYA8[2](0, 1) as u8);
gray!(rgb::alt::GRAYA16[2](0, 1) as u16);
