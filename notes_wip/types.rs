use std::path::Path;

// Hold the text range where the object is declared for error reporting.
#[derive(Debug, Clone)]
pub struct SourceTextOrigin {
    pub path: Box<Path>,
    pub begin_line: u64,
    pub begin_column: u64,
    pub end_line: u64,
    pub end_column: u64,
}

// 6.11 Integer data types (page 104)
// The term _integral_ is used throughout this standard to refer to the data
// types that can represent a single basic integer data type, packed array,
// packed structure, packed union, enum variable, or time variable.
// Types that can have unknown and high-impedance values are called 4-state
// types.
// These are `logic`, `reg`, `integer`, and `time`.
// The other types do not have unknown values and are called 2-state types, for
// example, `bit` and `int`.
// The difference between `int` and `integer` is that int is a 2-state type and
// integer is a 4-state type.
// The 4-state values have additional bits, which encode the X and Z states.
#[derive(Debug, Clone)]
pub struct SvTypeIntegral {
    pub origin: Option<SourceTextOrigin>,

    // None -> anonymous, Some -> named
    pub identifier: Option<String>,

    // False -> 2-state, True -> 4-state
    pub fourstate: bool,

    // False -> unsized, True -> sized.
    pub sized: bool,

    // False -> unsigned, True -> signed.
    pub signed: bool,

    // Dimensions/shape of object.
    // None -> scalar=[(0,0)], Some -> vector.
    // The `Option`s are for convenience when processing single-bit signals,
    // i.e. a single bit can be represented by `packed=None` and `unpacked=None`.
    // If `sized=True`, then `$bits` is calculated from dimensions.
    pub packed: Option<Vec<(u64, u64)>>,
    pub unpacked: Option<Vec<(u64, u64)>>,

    // None -> An object with a type but not a value, like an enum.
    // Some() -> Object's value is held in an array of usize integers.
    // Bits in the object correspond to bits in `data` such that:
    // - There may be unused/ignored bits in `data`, as determined by the
    //   values of `packed`.
    // - The LSB in `data` holds the object's LSB.
    // - Packed dimensions are adjacent.
    // - A packed element with size wider than usize is continued onto the next
    //   uzise.
    // - Packed dimensions are aligned to usize, i.e. each packed dimension
    //   begins on a new usize.
    // - If `fourstate=True`, a packed dimension is represented by pairs of
    //   adjacent usize with 0/1 on the least-significant usize corresponding
    //   to X/Z on the most-significant usize.
    // - Unpacked dimensions are aligned to usize, i.e. each unpacked dimension
    //   begins on a new usize.
    //
    // Example 1: twostate 1b: 1'b0 = False
    //   Some([0])
    //
    // Example 2: twostate 1b: 1'b1 = True
    //   Some([1])
    //
    // Example 3: fourstate 1b: 1'b0 = False
    //   Some([0,0])
    //
    // Example 4: fourstate 1b: 1'b1 = True
    //   Some([1,0])
    //
    // Example 5: fourstate 1b: 1'bX = Unknown
    //   Some([0,1])
    //
    // Example 6: fourstate 1b: 1'bZ = HighImpedence
    //   Some([1,1])
    //
    // Example 7: twostate packed 5b: 5'd5 = 5'b00101
    //   Some([5])
    //
    // Example 8: twostate packed 5b (where usize=32): 45'(5 << 32 + 123)
    //   Some([123, 5])
    //
    // Example 9: fourstate packed 5b: 5'd5 = 5'b01XZ0
    //   Some([0xA, 0x6])
    // Even-index usize holds 0 or 1 (01010) and odd-index usize decides
    // whether to translate 0/1 into X/Z (00110).
    //
    // Example 10: fourstate packed 5b (where usize=32): 45'(5'b01XZ0 << 32 + 123)
    //   Some([123, 0, 0xA, 0x6])
    //
    // Example 11: twostate packed 1b unpacked x5: {1'b1, 1'b1, 1'b0, 1'b0, 1'b0}
    //   Some([0, 0, 0, 1, 1])
    //
    // Example 12: fourstate packed 1b unpacked x5: {1'b0, 1'b1, 1'bX, 1'bZ, 1'b0}
    //   Some([0,0,  1,1,  0,1,  1,0,  0,0])
    // ->       0     Z     X     1     0
    // ->      ^     ^     ^     ^     ^    "True not False" OR "HighImpedence not Unknown"
    // ->        ^     ^     ^     ^     ^  "Not possible to represent in twostate"
    //
    // Example 13: twostate packed 45b unpacked x5 (where usize=32): {123, 456, 789, 101112, 131415}
    //   Some([131415,0, 101112,0, 789,0, 456,0, 123,0])
    // ->      ^         ^         ^      ^      ^      "Lowest 32b of 45b"
    // ->             ^         ^      ^      ^      ^  "Highest 13b of 45b"
    //
    // Example 14: fourstate packed 45b unpacked x5 (where usize=32): {123, 456, 45'Z, 777, 888}
    //   Some([888,0,0,0, 777,0,0,0, -1,-1,0xFFF1,0xFFF1, 456,0,0,0, 123,0,0,0])
    // ->      ^          ^          ^                    ^          ^          "Lowest 32b of 45b", "True not False" OR "HighImpedence not Unknown"
    // ->          ^          ^         ^                     ^          ^      "Lowest 32b of 45b", "Not possible to represent in twostate"
    // ->            ^          ^          ^                    ^          ^    "Highest 13b of 45b", "True not False" OR "HighImpedence not Unknown"
    // ->              ^          ^               ^               ^          ^  "Highest 13b of 45b", "Not possible to represent in twostate"
    pub value: Option<Vec<usize>>,
}

// 6.12 Real, shortreal, and realtime data types (page 105)
// The `real` data type is the same as a C `double`.
// The `shortreal` data type is the same as a C `float`.
// The `realtime` declarations shall be treated synonymously with `real`
// declarations and can be used interchangeably.
// Variables of these three types are collectively referred to as *real
// variables*.
#[derive(Debug, Clone)]
pub enum SvRealType {
    Real(Option<f64>),
    Realtime(Option<f64>),
    Shortreal(Option<f32>),
}
#[derive(Debug, Clone)]
pub struct SvTypeReal {
    pub origin: Option<SourceTextOrigin>,

    // None -> anonymous, Some -> named
    pub identifier: Option<String>,

    // None -> An object with a type but not a value, like an argument.
    pub value: Option<SvRealType>,
}

// 6.13 Void data type (page 105)
// The `void` data type represents nonexistent data.
// This type can be specified as the return type of functions to indicate no
// return value.
// This type can also be used for members of tagged unions (see 7.3.2).
#[derive(Debug, Clone)]
pub struct SvTypeVoid {
    pub origin: Option<SourceTextOrigin>,
}

// 6.14 Chandle data type (page 105)
// The chandle data type represents storage for pointers passed using the DPI
// (see Clause 35).
// The size of a value of this data type is platform dependent, but shall be at
// least large enough to hold a pointer on the machine on which the tool is
// running.
#[derive(Debug, Clone)]
pub struct SvTypeChandle {
    pub origin: Option<SourceTextOrigin>,
    pub value: usize,
}

// 6.15 Class (page 106)
// A class variable can hold a handle to a class object.
#[derive(Debug, Clone)]
pub struct SvTypeClass {
    pub origin: Option<SourceTextOrigin>,

    // None -> anonymous, Some -> named
    pub identifier: Option<String>,

    pub handle: u8 // TODO: Reference to something like SvClass.
}

// 6.16 String data type (page 106)
// The string data type is an ordered collection of characters.
// The length of a string variable is the number of characters in the
// collection.
// Variables of type string are dynamic as their length may vary during
// simulation.
// A single character of a string variable may be selected for reading or
// writing by indexing the variable.
// A single character of a string variable is of type byte.
#[derive(Debug, Clone)]
pub struct SvTypeString {
    pub origin: Option<SourceTextOrigin>,

    // None -> An object with a type but not a value, like an argument.
    pub value: Option<String>,
}

// 6.17 Event data type (page 112)
// The `event` data type provides a handle to a synchronization object.
// The object referenced by an event variable can be explicitly triggered and
// waited for.
// Furthermore, event variables have a persistent triggered state that lasts
// for the duration of the entire time step.
// Its occurrence can be recognized by using the event control syntax described
// in 9.4.2.
//
// 15.5 Named events (page 359)
// An identifier declared as an event data type is called a _named event_.
// A named event can be triggered explicitly.
// It can be used in an event expression to control the execution of procedural
// statements in the same manner as event controls described in 9.4.2.
// A named event may also be used as a handle assigned from another named
// event.
// A named event provides a handle to an underlying synchronization object.
// When a process waits for an event to be triggered, the process is put on a
// queue maintained within the synchronization object.
// Processes can wait for a named event to be triggered either via the `@`
// operator or by the use of the `wait()` construct to examine their triggered
// state.
#[derive(Debug, Clone)]
pub struct SvSynchronisationObject {
    pub queue: Vec<u8>, // TODO: Vec of reference to SvProcess to execute.
}
#[derive(Debug, Clone)]
pub struct SvTypeEvent {
    pub origin: Option<SourceTextOrigin>,

    // None -> anonymous, Some -> named
    pub identifier: Option<String>,

    pub sync_object: SvSynchronisationObject,
}

// 6.18 User-defined types (page 112)
// A typedef may be used to give a user-defined name to an existing data type.
//
// Sometimes a user-defined type needs to be declared before the contents of
// the type have been defined.
// This is of use with user-defined types derived from the basic data types:
// `enum`, `struct`, `union`, `interface class`, and `class`.
// Support for this is provided by the following forms for a _forward typedef_
// ...
// The actual data type definition of a forward `typedef` declaration shall be
// resolved within the same local scope or generate block.
// It shall be an error if the type_identifier does not resolve to a data type.
#[derive(Debug, Clone)]
pub struct SvTypeTypedef {
    pub origin: Option<SourceTextOrigin>,

    // None -> anonymous, Some -> named
    pub identifier: Option<String>,

    pub base_type: Box<SvType>,
}

// 6.19 Enumerations (page 114)
// An enumerated type declares a set of integral named constants.
// Enumerated data types provide the capability to abstractly declare strongly
// typed variables without either a data type or data value(s) and later add
// the required data type and value(s) for designs that require more
// definition.
// Enumerated data types also can be easily referenced or displayed using the
// enumerated names as opposed to the enumerated values.
//
// In the absence of a data type declaration, the default data type shall be
// `int`.
// Any other data type used with enumerated types shall require an explicit
// data type declaration.
#[derive(Debug, Clone)]
pub struct SvEnumMember {
    pub origin: Option<SourceTextOrigin>,

    pub identifier: String,

    pub value: Box<SvTypeIntegral>,
}
#[derive(Debug, Clone)]
pub struct SvTypeEnum {
    pub origin: Option<SourceTextOrigin>,

    // None -> anonymous, Some -> named
    pub identifier: Option<String>,

    pub base_type: Box<SvTypeIntegral>,

    pub members: Vec<SvEnumMember>,
}

#[derive(Debug, Clone)]
pub enum SvType {
    Integral(Box<SvTypeIntegral>),
    Real(Box<SvTypeReal>),
    Void(Box<SvTypeVoid>),
    Chandle(Box<SvTypeChandle>),
    Class(Box<SvTypeClass>),
    String(Box<SvTypeString>),
    Event(Box<SvTypeEvent>),
    Typedef(Box<SvTypeTypedef>),
    Enum(Box<SvTypeTypedef>),
}

// 6.22 Type compatibility (page 128)
// Some constructs and operations require a certain level of type compatibility
// for their operands to be legal.
// There are five levels of type compatibility, formally defined here:
// matching, equivalent, assignment compatible, cast compatible, and
// nonequivalent.
#[derive(Debug, Clone)]
pub enum SvTypesCompatibility {
    Matching,
    Equivalent,
    AssignmentCompatible,
    CastCompatible,
    NonEquivalent,
}

// TODO: 7 Aggregate data types
//   TODO: 7.2 Structures
//   TODO: 7.3 Unions
//   TODO: 7.4 Packed and unpacked arrays
//   TODO: 7.5 Dynamic arrays
//   TODO: 7.8 Associative arrays
//   TODO: 7.10 Queues
// TODO: 8 Classes

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
