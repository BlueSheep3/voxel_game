/// Similar to a match expression, but picks the branch with the lowest value in it.
///
/// The syntax for an expression without a code block is not supported.
///
/// # ParialOrd
///
/// This macro also works with types that implement `PartialOrd` but not `Ord`,
/// despite `min` only allowing for types that implement `Ord`. In cases where
/// `ParialOrd` gives an unconsistant result, it may be unclear which branch is picked.
///
/// # Code duplication
///
/// For best performance, this macro uses pure if else expressions to model this behaviour.
/// This means that some code will be duplicated, producing `2 ^ (n - 1)` code blocks.
/// Therefore, this should not be used with very many branches.
///
/// # Examples
///
/// ```
/// let x = match_min! {
///     3.48 => { 0 }
///     -1.2 => { 1 }
///     9.81 => { 2 }
/// };
/// assert_eq!(x, 1);
/// ```
#[macro_export]
macro_rules! match_min {
	($x:expr => $bx:block) => { $bx };
	($x:expr => $bx:block $y:expr => $by:block $($r:tt)*) => {
		if $x < $y {
			$crate::match_min!($x => $bx $($r)*)
		} else {
			$crate::match_min!($y => $by $($r)*)
		}
	};
}

/// Similar to a match expression, but picks the branch with the highest value in it.
///
/// The syntax for an expression without a code block is not supported.
///
/// # ParialOrd
///
/// This macro also works with types that implement `PartialOrd` but not `Ord`,
/// despite `max` only allowing for types that implement `Ord`. In cases where
/// `ParialOrd` gives an unconsistant result, it may be unclear which branch is picked.
///
/// # Code duplication
///
/// For best performance, this macro uses pure if else expressions to model this behaviour.
/// This means that some code will be duplicated, producing `2 ^ (n - 1)` code blocks.
/// Therefore, this should not be used with very many branches.
///
/// # Examples
///
/// ```
/// let x = match_max! {
///     3.48 => { 0 }
///     -1.2 => { 1 }
///     9.81 => { 2 }
/// };
/// assert_eq!(x, 2);
/// ```
#[macro_export]
macro_rules! match_max {
	($x:expr => $bx:block) => { $bx };
	($x:expr => $bx:block $y:expr => $by:block $($r:tt)*) => {
		if $x > $y {
			$crate::match_max!($x => $bx $($r)*)
		} else {
			$crate::match_max!($y => $by $($r)*)
		}
	};
}
