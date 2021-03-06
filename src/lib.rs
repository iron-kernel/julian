#![allow(dead_code)]
#![feature(const_fn)]
extern crate radish;

use std::fmt;
use std::cmp::Ordering;
use std::str::{self, FromStr};

use radish::err::ParseNumErr;
use radish::ascii::{FromBytes, isalnum, isalpha, isdigit, strtod, strtoi, tolower};



// Date Orders
#[derive(PartialEq)]
pub enum DateOrder {
  YMD,
  DMY,
  MDY
}

pub static DATE_ORDER: DateOrder = DateOrder::YMD;

// ---------------------------------------------------------------------------
// Ported from datetime.h
// ---------------------------------------------------------------------------


// ----------------------------------------------------------------
//              time types + support macros
//
// String definitions for standard time quantities.
//
// These strings are the defaults used to form output time strings.
// Other alternative forms are hardcoded into token tables in datetime.c.
// ----------------------------------------------------------------
const DAGO       :&'static [u8] = b"ago";
const DCURRENT   :&'static [u8] = b"current";
const EPOCH      :&'static [u8] = b"epoch";
const INVALID    :&'static [u8] = b"invalid";
const EARLY      :&'static [u8] = b"-infinity";
const LATE       :&'static [u8] = b"infinity";
const NOW        :&'static [u8] = b"now";
const TODAY      :&'static [u8] = b"today";
const TOMORROW   :&'static [u8] = b"tomorrow";
const YESTERDAY  :&'static [u8] = b"yesterday";
const ZULU       :&'static [u8] = b"zulu";

const DMICROSEC  :&'static [u8] = b"usecond";
const DMILLISEC  :&'static [u8] = b"msecond";
const DSECOND    :&'static [u8] = b"second";
const DMINUTE    :&'static [u8] = b"minute";
const DHOUR      :&'static [u8] = b"hour";
const DDAY       :&'static [u8] = b"day";
const DWEEK      :&'static [u8] = b"week";
const DMONTH     :&'static [u8] = b"month";
const DQUARTER   :&'static [u8] = b"quarter";
const DYEAR      :&'static [u8] = b"year";
const DDECADE    :&'static [u8] = b"decade";
const DCENTURY   :&'static [u8] = b"century";
const DMILLENNIUM:&'static [u8] = b"millennium";
const DA_D       :&'static [u8] = b"ad";
const DB_C       :&'static [u8] = b"bc";
const DTIMEZONE  :&'static [u8] = b"timezone";

// Fundamental time field definitions for parsing.
//
// Meridian:  am, pm, or 24-hour style.
// Millennium: ad, bc
const AM   :i32 = 0;
const PM   :i32 = 1;
const HR24 :i32 = 2;

const AD   :i32 = 0;
const BC   :i32 = 1;

// Field types for time decoding.
//
// Can't have more of these than there are bits in an unsigned int
// since these are turned into bit masks during parsing and decoding.
//
// Furthermore, the values for YEAR, MONTH, DAY, HOUR, MINUTE, SECOND
// must be in the range 0..14 so that the associated bitmasks can fit
// into the left half of an INTERVAL's typmod value.  Since those bits
// are stored in typmods, you can't change them without initdb!

const RESERV        :i8 = 0;
const MONTH         :i8 = 1;
const YEAR          :i8 = 2;
const DAY           :i8 = 3;
const JULIAN        :i8 = 4;
/// fixed-offset timezone abbreviation
const TZ            :i8 = 5;
/// fixed-offset timezone abbrev, DST
const DTZ           :i8 = 6;
/// dynamic timezone abbreviation
const DYNTZ         :i8 = 7;
const IGNORE_DTF    :i8 = 8;
const AMPM          :i8 = 9;
const HOUR          :i8 = 10;
const MINUTE        :i8 = 11;
const SECOND        :i8 = 12;
const MILLISECOND   :i8 = 13;
const MICROSECOND   :i8 = 14;
const DOY           :i8 = 15;
const DOW           :i8 = 16;
const UNITS         :i8 = 17;
const ADBC          :i8 = 18;
const AGO           :i8 = 19; /// these are only for relative dates
const ABS_BEFORE    :i8 = 20;
const ABS_AFTER     :i8 = 21;
const ISODATE       :i8 = 22; // generic fields to help with parsing
const ISOTIME       :i8 = 23;
const WEEK          :i8 = 24; // these are only for parsing intervals
const DECADE        :i8 = 25;
const CENTURY       :i8 = 26;
const MILLENNIUM    :i8 = 27;
/// hack for parsing two-word timezone specs "MET DST" etc
const DTZMOD        :i8 = 28; // "DST" as a separate word
/// reserved for unrecognized string values
const UNKNOWN_FIELD :i8 = 31;



// Token field definitions for time parsing and decoding.
//
// Some field type codes (see above) use these as the "value" in DATETK_TBL[].
// These are also used for bit masks in DecodeDateTime and friends
//  so actually restrict them to within [0,31] for now.
// - thomas 97/06/19
// Not all of these fields are used for masks in DecodeDateTime
//  so allow some larger than 31. - thomas 1997-11-17
//
// Caution: there are undocumented assumptions in the code that most of these
// values are not equal to IGNORE_DTF nor RESERV.  Be very careful when
// renumbering values in either of these apparently-independent lists :-(
const DTK_NUMBER     :i32 = 0;
const DTK_STRING     :i32 = 1;

const DTK_DATE       :i32 = 2;
const DTK_TIME       :i32 = 3;
const DTK_TZ         :i32 = 4;
const DTK_AGO        :i32 = 5;

const DTK_SPECIAL    :i32 = 6;
const DTK_INVALID    :i32 = 7;
const DTK_CURRENT    :i32 = 8;
const DTK_EARLY      :i32 = 9;
const DTK_LATE       :i32 = 10;
const DTK_EPOCH      :i32 = 11;
const DTK_NOW        :i32 = 12;
const DTK_YESTERDAY  :i32 = 13;
const DTK_TODAY      :i32 = 14;
const DTK_TOMORROW   :i32 = 15;
const DTK_ZULU       :i32 = 16;

const DTK_DELTA      :i32 = 17;
const DTK_SECOND     :i32 = 18;
const DTK_MINUTE     :i32 = 19;
const DTK_HOUR       :i32 = 20;
const DTK_DAY        :i32 = 21;
const DTK_WEEK       :i32 = 22;
const DTK_MONTH      :i32 = 23;
const DTK_QUARTER    :i32 = 24;
const DTK_YEAR       :i32 = 25;
const DTK_DECADE     :i32 = 26;
const DTK_CENTURY    :i32 = 27;
const DTK_MILLENNIUM :i32 = 28;
const DTK_MILLISEC   :i32 = 29;
const DTK_MICROSEC   :i32 = 30;
const DTK_JULIAN     :i32 = 31;

const DTK_DOW        :i32 = 32;
const DTK_DOY        :i32 = 33;
const DTK_TZ_HOUR    :i32 = 34;
const DTK_TZ_MINUTE  :i32 = 35;
const DTK_ISOYEAR    :i32 = 36;
const DTK_ISODOW     :i32 = 37;

#[allow(non_snake_case)]
const fn DTK_M(token: i8) -> i32 {
  (0x01 << token)
}

const DTK_ALL_SECS_M   :i32 = (DTK_M(SECOND) | DTK_M(MILLISECOND) | DTK_M(MICROSECOND));
const DTK_YEAR_M       :i32 = DTK_M(YEAR);
const DTK_MONTH_M      :i32 = DTK_M(MONTH);
const DTK_DAY_M        :i32 = DTK_M(DAY);
const DTK_YEAR_MONTH_M :i32 = (DTK_M(YEAR) | DTK_M(MONTH));
const DTK_MONTH_DAY_M  :i32 = (DTK_M(MONTH) | DTK_M(DAY));
const DTK_DATE_M       :i32 = (DTK_M(YEAR) | DTK_M(MONTH) | DTK_M(DAY));
const DTK_TIME_M       :i32 = (DTK_M(HOUR) | DTK_M(MINUTE) | DTK_ALL_SECS_M);

/* maximum possible number of fields in a date string */
const MAXDATEFIELDS  :usize	= 25;

// ---------------------------------------------------------------------------
// Ported from Timestamp.h
// ---------------------------------------------------------------------------

pub type Timestamp   = i64;
pub type TimestampTz = i64;
pub type TimeOffset  = i64;
pub type FracSec     = i32;

pub struct Interval {
  time: TimeOffset,
  day: i32,
  month: i32
}

const MAX_TIMESTAMP_PRECISION :i32 = 6;
const MAX_INTERVAL_PRECISION  :i32 = 6;


// Assorted constants for datetime-related calculations
const DAYS_PER_YEAR    :f32 = 365.25; // assumes leap year every four years
const MONTHS_PER_YEAR  :i32  = 12;

// DAYS_PER_MONTH is very imprecise.  The more accurate value is
// 365.2425/12 = 30.436875, or '30 days 10:29:06'.  Right now we only
// return an integral number of days, but someday perhaps we should
// also return a 'time' value to be used as well.  ISO 8601 suggests
// 30 days.
const DAYS_PER_MONTH   :i32 = 30; // assumes exactly 30 days per month
const HOURS_PER_DAY    :i32 = 24; // assume no daylight savings time changes

// This doesn't adjust for uneven daylight savings time intervals or leap
// seconds, and it crudely estimates leap years.  A more accurate value
// for days per years is 365.2422.
const SECS_PER_YEAR    :i32 = (36525 * 864); /* avoid floating-point computation */
const SECS_PER_DAY     :i32 = 86400;
const SECS_PER_HOUR    :i32 = 3600;
const SECS_PER_MINUTE  :i32 = 60;
const MINS_PER_HOUR    :i32 = 60;

const USECS_PER_DAY    :i64 = 86400000000;
const USECS_PER_HOUR   :i64 = 3600000000;
const USECS_PER_MINUTE :i64 = 60000000;
const USECS_PER_SEC    :i64 = 1000000;

/// maximum allowed hour part
const MAX_TZDISP_HOUR  : i32 = 15;

// ---------------------------------------------------------------------------
// Ported from pgtime.h
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct TimeMeta {
  tm_sec: i32,
	tm_min: i32,
	tm_hour: i32,
	tm_mday: i32,
  /// origin 0, not 1
	tm_mon: i32,
  /// relative to 1900
	tm_year: i32,
	tm_wday: i32,
	tm_yday: i32,
	tm_isdst: i32,
	tm_gmtoff: i64,
	tm_zone: Option<&'static str>
}

impl TimeMeta {
  pub fn empty() -> TimeMeta {
    TimeMeta {
      tm_sec: 0,
      tm_min: 0,
      tm_hour: 0,
      tm_mday: 0,
      tm_mon: 0,
      tm_year: 0,
      tm_wday: 0,
      tm_yday: 0,
      tm_isdst: 0,
      tm_gmtoff: 0,
      tm_zone: None
    }
  }
}

// ---------------------------------------------------------------------------
// Ported from datetime.c
// ---------------------------------------------------------------------------

const DAY_TAB: [[i32;13];2] = [
  [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31, 0],
  [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31, 0]
];

// Removed NULL
const MONTHS: [&'static str;12] = [
   "Jan", "Feb", "Mar", "Apr", "May", "Jun",
   "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

const DAYS: [&'static str;7] = [
  "Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"
];

pub struct DateToken {
  token: &'static [u8],
  ty: i8,
  value: i32
}

macro_rules! token {
  ($token:expr, $ty:expr, $value:expr) => {
    DateToken {
      token: $token,
      ty: $ty,
      value: $value
    }
  }
}

pub static DATETK_TBL: [DateToken;74] = [
  token!(EARLY, RESERV, DTK_EARLY),
  token!(DA_D, ADBC, AD),                     // "ad" for years > 0
  token!(b"allballs", RESERV, DTK_ZULU),      // 00:00:00
  token!(b"am", AMPM, AM),
  token!(b"apr", MONTH, 4),
  token!(b"april", MONTH, 4),
  token!(b"at", IGNORE_DTF, 0),               // "at" (throwaway)
  token!(b"aug", MONTH, 8),
  token!(b"august", MONTH, 8),
  token!(DB_C, ADBC, BC),                     // "bc" for years <= 0
  token!(DCURRENT, RESERV, DTK_CURRENT),      // "current" is always now
  token!(b"d", UNITS, DTK_DAY),               // "day of month" for ISO input
  token!(b"dec", MONTH, 12),
  token!(b"december", MONTH, 12),
  token!(b"dow", RESERV, DTK_DOW),            // day of week
  token!(b"doy", RESERV, DTK_DOY),            // day of year
  token!(b"dst", DTZMOD, SECS_PER_HOUR),
  token!(EPOCH, RESERV, DTK_EPOCH),           // "epoch" reserved for system epoch time
  token!(b"feb", MONTH, 2),
  token!(b"february", MONTH, 2),
  token!(b"fri", DOW, 5),
  token!(b"friday", DOW, 5),
  token!(b"h", UNITS, DTK_HOUR),              // "hour"
  token!(LATE, RESERV, DTK_LATE),             // "infinity" reserved for "late time"
  token!(INVALID, RESERV, DTK_INVALID),       // "invalid" reserved for bad time
  token!(b"isodow", RESERV, DTK_ISODOW),      // ISO day of week, Sunday == 7
  token!(b"isoyear", UNITS, DTK_ISOYEAR),     // year in terms of the ISO week date
  token!(b"j", UNITS, DTK_JULIAN),
  token!(b"jan", MONTH, 1),
  token!(b"january", MONTH, 1),
  token!(b"jd", UNITS, DTK_JULIAN),
  token!(b"jul", MONTH, 7),
  token!(b"julian", UNITS, DTK_JULIAN),
  token!(b"july", MONTH, 7),
  token!(b"jun", MONTH, 6),
  token!(b"june", MONTH, 6),
  token!(b"m", UNITS, DTK_MONTH),              // "month" for ISO input
  token!(b"mar", MONTH, 3),
  token!(b"march", MONTH, 3),
  token!(b"may", MONTH, 5),
  token!(b"mm", UNITS, DTK_MINUTE),            // "minute" for ISO input
  token!(b"mon", DOW, 1),
  token!(b"monday", DOW, 1),
  token!(b"nov", MONTH, 11),
  token!(b"november", MONTH, 11),
  token!(NOW, RESERV, DTK_NOW),                // current transaction time
  token!(b"oct", MONTH, 10),
  token!(b"october", MONTH, 10),
  token!(b"on", IGNORE_DTF, 0),                // "on" (throwaway)
  token!(b"pm", AMPM, PM),
  token!(b"s", UNITS, DTK_SECOND),             // "seconds" for ISO input
  token!(b"sat", DOW, 6),
  token!(b"saturday", DOW, 6),
  token!(b"sep", MONTH, 9),
  token!(b"sept", MONTH, 9),
  token!(b"september", MONTH, 9),
  token!(b"sun", DOW, 0),
  token!(b"sunday", DOW, 0),
  token!(b"t", ISOTIME, DTK_TIME),             // Filler for ISO time fields
  token!(b"thu", DOW, 4),
  token!(b"thur", DOW, 4),
  token!(b"thurs", DOW, 4),
  token!(b"thursday", DOW, 4),
  token!(TODAY, RESERV, DTK_TODAY),            // midnight
  token!(TOMORROW, RESERV, DTK_TOMORROW),      // tomorrow midnight
  token!(b"tue", DOW, 2),
  token!(b"tues", DOW, 2),
  token!(b"tuesday", DOW, 2),
  token!(b"undefined", RESERV, DTK_INVALID),   // pre-v6.1 invalid time
  token!(b"wed", DOW, 3),
  token!(b"wednesday", DOW, 3),
  token!(b"weds", DOW, 3),
  token!(b"y", UNITS, DTK_YEAR),               // "year" for ISO input
  token!(YESTERDAY, RESERV, DTK_YESTERDAY)     // yesterday midnight
];


static DELTATK_TBL: [DateToken;63] = [
  token!(b"@", IGNORE_DTF, 0),                 // postgres relative prefix
  token!(DAGO, AGO, 0),                        // "ago" indicates negative time offset
  token!(b"c", UNITS, DTK_CENTURY),            // "century" relative
  token!(b"cent", UNITS, DTK_CENTURY),         // "century" relative
  token!(b"centuries", UNITS, DTK_CENTURY),    // "centuries" relative
  token!(DCENTURY, UNITS, DTK_CENTURY),        // "century" relative
  token!(b"d", UNITS, DTK_DAY),                // "day" relative
  token!(DDAY, UNITS, DTK_DAY),                // "day" relative
  token!(b"days", UNITS, DTK_DAY),             // "days" relative
  token!(b"dec", UNITS, DTK_DECADE),           // "decade" relative
  token!(DDECADE, UNITS, DTK_DECADE),          // "decade" relative
  token!(b"decades", UNITS, DTK_DECADE),       // "decades" relative
  token!(b"decs", UNITS, DTK_DECADE),          // "decades" relative
  token!(b"h", UNITS, DTK_HOUR),               // "hour" relative
  token!(DHOUR, UNITS, DTK_HOUR),              // "hour" relative
  token!(b"hours", UNITS, DTK_HOUR),           // "hours" relative
  token!(b"hr", UNITS, DTK_HOUR),              // "hour" relative
  token!(b"hrs", UNITS, DTK_HOUR),             // "hours" relative
  token!(INVALID, RESERV, DTK_INVALID),        // reserved for invalid time
  token!(b"m", UNITS, DTK_MINUTE),             // "minute" relative
  token!(b"microsecon", UNITS, DTK_MICROSEC),  // "microsecond" relative
  token!(b"mil", UNITS, DTK_MILLENNIUM),       // "millennium" relative
  token!(b"millennia", UNITS, DTK_MILLENNIUM), // "millennia" relative
  token!(DMILLENNIUM, UNITS, DTK_MILLENNIUM),  // "millennium" relative
  token!(b"millisecon", UNITS, DTK_MILLISEC),  // relative
  token!(b"mils", UNITS, DTK_MILLENNIUM),      // "millennia" relative
  token!(b"min", UNITS, DTK_MINUTE),           // "minute" relative
  token!(b"mins", UNITS, DTK_MINUTE),          // "minutes" relative
  token!(DMINUTE, UNITS, DTK_MINUTE),          // "minute" relative
  token!(b"minutes", UNITS, DTK_MINUTE),       // "minutes" relative
  token!(b"mon", UNITS, DTK_MONTH),            // "months" relative
  token!(b"mons", UNITS, DTK_MONTH),           // "months" relative
  token!(DMONTH, UNITS, DTK_MONTH),            // "month" relative
  token!(b"months", UNITS, DTK_MONTH),
  token!(b"ms", UNITS, DTK_MILLISEC),
  token!(b"msec", UNITS, DTK_MILLISEC),
  token!(DMILLISEC, UNITS, DTK_MILLISEC),
  token!(b"mseconds", UNITS, DTK_MILLISEC),
  token!(b"msecs", UNITS, DTK_MILLISEC),
  token!(b"qtr", UNITS, DTK_QUARTER),          // "quarter" relative
  token!(DQUARTER, UNITS, DTK_QUARTER),        // "quarter" relative
  token!(b"s", UNITS, DTK_SECOND),
  token!(b"sec", UNITS, DTK_SECOND),
  token!(DSECOND, UNITS, DTK_SECOND),
  token!(b"seconds", UNITS, DTK_SECOND),
  token!(b"secs", UNITS, DTK_SECOND),
  token!(DTIMEZONE, UNITS, DTK_TZ),            // "timezone" time offset
  token!(b"timezone_h", UNITS, DTK_TZ_HOUR),   // timezone hour units
  token!(b"timezone_m", UNITS, DTK_TZ_MINUTE), // timezone minutes units
  token!(b"undefined", RESERV, DTK_INVALID),   // pre-v6.1 invalid time
  token!(b"us", UNITS, DTK_MICROSEC),          // "microsecond" relative
  token!(b"usec", UNITS, DTK_MICROSEC),        // "microsecond" relative
  token!(DMICROSEC, UNITS, DTK_MICROSEC),      // "microsecond" relative
  token!(b"useconds", UNITS, DTK_MICROSEC),    // "microseconds" relative
  token!(b"usecs", UNITS, DTK_MICROSEC),       // "microseconds" relative
  token!(b"w", UNITS, DTK_WEEK),               // "week" relative
  token!(DWEEK, UNITS, DTK_WEEK),              // "week" relative
  token!(b"weeks", UNITS, DTK_WEEK),           // "weeks" relative
  token!(b"y", UNITS, DTK_YEAR),               // "year" relative
  token!(DYEAR, UNITS, DTK_YEAR),              // "year" relative
  token!(b"years", UNITS, DTK_YEAR),           // "years" relative
  token!(b"yr", UNITS, DTK_YEAR),              // "year" relative
  token!(b"yrs", UNITS, DTK_YEAR)              // "years" relative
];


/// Calendar time to Julian date conversions.
/// Julian date is commonly used in astronomical applications,
///  since it is numerically accurate and computationally simple.
/// The algorithms here will accurately convert between Julian day
///  and calendar date for all non-negative Julian days
///  (i.e. from Nov 24, -4713 on).
///
/// These routines will be used by other date/time packages
/// - thomas 97/02/25
///
/// Rewritten to eliminate overflow problems. This now allows the
/// routines to work correctly for all Julian day counts from
/// 0 to 2147483647  (Nov 24, -4713 to Jun 3, 5874898) assuming
/// a 32-bit integer. Longer types should also work to the limits
/// of their precision.
pub fn date2j(mut y: i32, mut m: i32, d: i32) -> i32 {
  if m > 2 {
      m += 1;
      y += 4800;
  } else {
      m += 13;
      y += 4799;
  }

  let century: i32 = y / 100;
  let mut julian: i32 = y * 365 - 32167;
  julian += y / 4 - century + century / 4;
  julian += 7834 * m / 256 + d;

  julian
}

fn j2date(julian_day: u32) -> (i32, u32, u32) {
  let mut julian: u32 = julian_day;
  julian += 32044;
  let mut quad: u32 = julian / 146097;
  let extra: u32 = (julian - quad * 146097) * 4 + 3;
  julian += 60 + quad * 3 + extra / 146097;
  quad = julian / 1461;
  julian -= quad * 1461;
  let mut y: u32 = julian * 4 / 1461;

  julian = if y != 0 {
    ((julian + 305) % 365)
  } else {
    ((julian + 306) % 366) + 123
  };

  y += quad * 4;
  let year :i32 = (y - 4800) as i32;
  quad = julian * 2141 / 65536;
  let day: u32 = julian - 7834 * quad / 256;
  let month: u32 = (quad + 10) % MONTHS_PER_YEAR as u32 + 1;

  (year, month, day)
}


/// j2day - convert Julian date to day-of-week (0..6 == Sun..Sat)
///
/// Note: various places use the locution j2day(date - 1) to produce a
/// result according to the convention 0..6 = Mon..Sun.  This is a bit of
/// a crock, but will work as long as the computation here is just a modulo.
pub fn j2day(mut date: i32) -> i32 {
  date += 1;
  date %= 7;

  if date < 0 {
    date += 7;
  }

  date
}

#[derive(PartialEq, Eq)]
pub enum DateTimeParseError {
  BadFormat(String),
  TimezoneOverflow
}

impl fmt::Debug for DateTimeParseError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      DateTimeParseError::BadFormat(ref s) => write!(f, "{}", s),
      DateTimeParseError::TimezoneOverflow => {
        write!(f, "overflow or underflow in timezone")
      }
    }
  }
}

impl From<ParseNumErr> for DateTimeParseError {
  fn from(e: ParseNumErr) -> Self {
    DateTimeParseError::BadFormat(format!("{}", e))
  }
}

/// Parse a string to a fractional second.
pub fn parse_fractional_second(s: &str) -> Result<i64, DateTimeParseError> {
  debug_assert!(s.len() > 1);
  debug_assert!(s.as_bytes()[0] == b'.');

  let part = &s[1..];
  match i64::from_str(part) {
    Ok(frac) => Ok(frac * 1000000),
    Err(e) => Err(DateTimeParseError::BadFormat(format!("{}: '{}'", e, s)))
  }
}

/// Decode date string which includes delimiters.
/// Return () if okay, a DateTimeParseError if not.
/// * str: field to be parsed
/// * fmask: bitmask for field types already seen
/// * tmask: receives bitmask for fields found here
/// * is2digits: set to TRUE if we find 2-digit year
/// * tm: field values are stored into appropriate members of this struct
pub fn decode_date(s: &[u8], fmask: &mut i32, tmask: &mut i32, is2digits: &mut bool, 
    tm: &mut TimeMeta) -> Result<(), DateTimeParseError> {

  let len = s.len();
  let mut idx = 0;
  let mut dmask: i32 = 0;
  let mut fields: Vec<&[u8]> = Vec::with_capacity(MAXDATEFIELDS);
  let mut has_text_month = false;
  let mut fields_identified: [bool;8] = unsafe { ::std::mem::zeroed() };

  // parse this string...
  while idx < len && fields.len() < MAXDATEFIELDS {

    // skip field separators
    while idx < len && !isalnum(s[idx]) {
      idx += 1;
    }

    if idx == len {
      return Err(DateTimeParseError::BadFormat(format!("bad date format: '{}'",
        unsafe { str::from_utf8_unchecked(s) })));
    }

    let field_start_idx = idx;
    if isdigit(s[idx]) {
      while idx < len && isdigit(s[idx]) {
        idx += 1;
      }
    } else if isalpha(s[idx]) {
      while idx < len && isalpha(s[idx]) {
        idx += 1;
      }
    }
    fields.push(&s[field_start_idx .. idx]);
  }

  for i in 0..fields.len() {
    println!("fields[{}]=\"{}\"", i, unsafe { str::from_utf8_unchecked(fields[i]) });
  }

  // look first for text fields, since that will be unambiguous month
  for i in 0..fields.len() {
     
     if isalpha(fields[i][0]) {       
       
       if let Some(datetk) = datebsearch(tolower(fields[i]).as_slice(), &DATETK_TBL) {
         let ty = datetk.ty;         
         if ty == IGNORE_DTF {
           continue;
         }

         dmask = DTK_M(ty);
         match ty {
           MONTH => {
             tm.tm_mon = datetk.value;
             has_text_month = true;             
           }
           _ => {
             return Err(DateTimeParseError::BadFormat(format!("1 bad date format: '{}'",
              unsafe { str::from_utf8_unchecked(s) })));
           }
         };

         if (*fmask & dmask) != 0 {
				  return Err(DateTimeParseError::BadFormat(format!("2 bad date format: '{}'",
              unsafe { str::from_utf8_unchecked(s) })));
         }

         *fmask = *fmask | dmask;
			   *tmask = *tmask | dmask;
         
       } else {
         return Err(DateTimeParseError::BadFormat(format!("3 bad date format: '{}'",
          unsafe { str::from_utf8_unchecked(s) })));
       }

       fields_identified[i] = true;
     }
  }

  for i in 0..fields.len() {
    println!("all fields parsed...");
  }
  for i in 0..fields.len() {
    if fields_identified[i] {
      println!("fields_identified[{}]=true", i);
      continue;
    } else {
      println!("fields_identified[{}]=false", i);
    }

    let len = fields[i].len();
    if len <= 0 {
      return Err(DateTimeParseError::BadFormat(format!("bad date format: '{}'",
          unsafe { str::from_utf8_unchecked(s) })));
    }

    decode_number(len, fields[i], has_text_month, fmask, &mut dmask, tm, &mut 0, is2digits)?;

    if (*fmask & dmask) != 0 {
			return Err(DateTimeParseError::BadFormat(format!("bad date format: '{}'",
          unsafe { str::from_utf8_unchecked(s) })));
    }

		*fmask = *fmask | dmask;
		*tmask = *tmask | dmask;
  }

  if (*fmask & !(DTK_M(DOY) | DTK_M(TZ))) != DTK_DATE_M {
    return Err(DateTimeParseError::BadFormat(format!("bad date format: '{}'",
          unsafe { str::from_utf8_unchecked(s) })));
  }

  Ok(())
}

/// Interpret plain numeric field as a date value in context.
/// Return () if okay, a DateTimeParseError code if not.
fn decode_number(flen: usize, s: &[u8], has_text_month: bool, fmask: &mut i32,
    tmask: &mut i32, tm: &mut TimeMeta, fsec: &mut FracSec, is2digits: &mut bool)
    -> Result<(), DateTimeParseError> {

  let (val, remain) = unsafe { strtoi(s)? };

  if remain.is_some() && remain.unwrap()[0] == b'.' {
    let remain = remain.unwrap();

    if (s.len() - remain.len()) > 2 {
      decode_number_field(flen, s, (*fmask | DTK_DATE_M), tmask, tm, fsec, is2digits)?;
      return Ok(());
    }

  } else if remain.is_some() {
    return Err(DateTimeParseError::BadFormat(
      format!("invalid number format: '{}'",
        unsafe { str::from_utf8_unchecked(s) } )));
  }

  // Special case for day of year
  if flen == 3 && (*fmask & DTK_DATE_M) == DTK_M(YEAR) &&
     val > 1 && val <= 366 {
    *tmask = (DTK_M(DOY) | DTK_M(MONTH) | DTK_M(DAY));
    tm.tm_yday = val;
    return Ok(())
  }

  /* Switch based on what we have so far */
	match *fmask & DTK_DATE_M {
    0 => {
      
			 // Nothing so far; make a decision about what we think the input
			 // is. There used to be lots of heuristics here, but the
			 // consensus now is to be paranoid.  It *must* be either
			 // YYYY-MM-DD (with a more-than-two-digit year field), or the
			 // field order defined by DATE_ORDER.
       if flen >= 3 {
         *tmask = DTK_M(YEAR);
         tm.tm_year = val;
       } else {
         match DATE_ORDER {
           DateOrder::YMD => {
             *tmask = DTK_M(YEAR);
             tm.tm_year = val;
           }
           DateOrder::DMY => {
             *tmask = DTK_M(DAY);
             tm.tm_mday = val;
           }
           DateOrder::MDY => {
             *tmask = DTK_M(MONTH);
             tm.tm_mon = val;
           }
         } 
       }       			
    }
    DTK_YEAR_M => {
      // Must be at second field of YY-MM-DD
      *tmask = DTK_M(MONTH);
			tm.tm_mon = val;
    }
    DTK_MONTH_M => {
      if has_text_month {
			  // We are at the first numeric field of a date that included a
			  // textual month name.  We want to support the variants
			  // MON-DD-YYYY, DD-MON-YYYY, and YYYY-MON-DD as unambiguous
			  // inputs.  We will also accept MON-DD-YY or DD-MON-YY in
			  // either DMY or MDY modes, as well as YY-MON-DD in YMD mode.
			  if flen >= 3 || DATE_ORDER == DateOrder::YMD {
          *tmask = DTK_M(YEAR);
          tm.tm_year = val;
        } else {
          *tmask = DTK_M(DAY);
          tm.tm_mday = val;
        }
      } else {
        // Must be at second field of MM-DD-YY
				*tmask = DTK_M(DAY);
				tm.tm_mday = val;
      }
    }
    DTK_YEAR_MONTH_M => {
      if has_text_month {
				// Need to accept DD-MON-YYYY even in YMD mode
				if flen >= 3 && *is2digits {
					// Guess that first numeric field is day was wrong
          *tmask = DTK_M(DAY);		// YEAR is already set
          tm.tm_mday = tm.tm_year;
          tm.tm_year = val;
          *is2digits = false;
			  } else {
			    *tmask = DTK_M(DAY);
			    tm.tm_mday = val;
			  }
			} else {
				// Must be at third field of YY-MM-DD
        *tmask = DTK_M(DAY);
        tm.tm_mday = val;
			}
    }
    DTK_DAY_M => {
      // Must be at second field of DD-MM-YY
			*tmask = DTK_M(MONTH);
			tm.tm_mon = val;
    }
    DTK_MONTH_DAY_M => {
      // Must be at third field of DD-MM-YY or MM-DD-YY
			*tmask = DTK_M(YEAR);
			tm.tm_year = val;
    }
    DTK_DATE_M => {
      // we have all the date, so it must be a time field
			decode_number_field(flen, s, *fmask, tmask, tm, fsec, is2digits)?;
      return Ok(())
    }
    _ => {
      return Err(DateTimeParseError::BadFormat(format!("bad date format: '{}'",
              unsafe { str::from_utf8_unchecked(s) })));
    }
  };

	 // When processing a year field, mark it for adjustment if it's only one
	 // or two digits.
	if *tmask == DTK_M(YEAR) {
		*is2digits = flen <= 2;
  }

  Ok(())
}

/// decode_number_field()
///
/// Interpret numeric string as a concatenated date or time field.
/// Return a DTK token if successful, a DateTimeParseError if error.
///
/// Use the context of previously decoded fields to help with
/// the interpretation.
fn decode_number_field(mut len: usize, s: &[u8], fmask: i32, tmask: &mut i32,
                      tm: &mut TimeMeta, fsec: &mut FracSec,
                      is2digits: &mut bool) -> Result<i32, DateTimeParseError> {

  // Have a decimal point? Then this is a date or something with a seconds
	// field...
  let decimal_point_idx = s.iter().position(|&c| c == b'.');
  if let Some(idx) = decimal_point_idx {
		 // Can we use ParseFractionalSecond here?  Not clear whether trailing
		 // junk should be rejected ...
     let (frac, remaion) = unsafe { strtod(&s[idx..])? };
     *fsec = (frac * 1000000f64).round() as i32;
     /* Now truncate off the fraction for further processing */
     len = idx - 1;

  // No decimal point and no complete date yet?
  } else if (fmask & DTK_DATE_M) != DTK_DATE_M {

    // yyyymmdd or yymmdd
    if len >= 6 {
      *tmask = DTK_DATE_M;
      tm.tm_mon = unsafe { i32::from_bytes(&s[(len - 2)..])? };
      tm.tm_mday = unsafe { i32::from_bytes(&s[(len - 4)..(len - 2)])? };
      tm.tm_year = unsafe { i32::from_bytes(&s[..(len-4)])? };

      if (len - 4) == 2 {
        *is2digits = true;
      }

      return Ok(DTK_DATE);
    }
  }

  /* not all time fields are specified? */
  if fmask & DTK_TIME_M != DTK_TIME_M {

		if len == 6 { /* hhmmss */
      *tmask = DTK_TIME_M;
      tm.tm_sec = unsafe { i32::from_bytes(&s[4..])? };
      tm.tm_min = unsafe { i32::from_bytes(&s[2..4])? };
      tm.tm_hour = unsafe { i32::from_bytes(&s[0..2])? };

      return Ok(DTK_TIME);

    } else if len == 4 { /* hhmm? */
      *tmask = DTK_TIME_M;
			tm.tm_sec = 0;
			tm.tm_min = unsafe { i32::from_bytes(&s[2..])? };
			tm.tm_hour = unsafe { i32::from_bytes(&s[..2])? };

      return Ok(DTK_TIME)
    }
  }

  Err(DateTimeParseError::BadFormat(format!("")))
}

/// Parse a string to a timezone in seconds.
pub fn decode_timezone(tzstr: &str) -> Result<i32, DateTimeParseError> {
  let buf = tzstr.as_bytes();
  let mut hr: i32;
  let min;
  let mut remains;
  let mut sec = 0;

  let plus_or_minus = buf[0];
  if plus_or_minus != b'+' && plus_or_minus != b'-' {
    return Err(DateTimeParseError::BadFormat(
      format!("leading characer in timezone must be '+' or '-': '{}'", tzstr)));
  }

  let r = unsafe { strtoi(&buf[1..])? };
  hr = r.0;
  remains = r.1;

  if remains.is_some() && remains.unwrap()[0] == b':' {
    let r = unsafe { strtoi(&remains.unwrap()[1..])? };
    min = r.0;
    remains = r.1;

    if remains.is_some() && remains.unwrap()[0] == b':' {
      let r = unsafe { strtoi(&remains.unwrap()[1..])? };
      sec = r.0;
      remains = r.1;
    }
  } else if remains.is_none() && buf.len() > 3 {
    min = hr % 100;
    hr = hr / 100;
  } else {
    min = 0;
  }

  if hr < 0 || hr > MAX_TZDISP_HOUR {
    return Err(DateTimeParseError::TimezoneOverflow);
  }
  if min < 0 || min >= MINS_PER_HOUR {
    return Err(DateTimeParseError::TimezoneOverflow)
  }
  if sec < 0 || sec >= SECS_PER_MINUTE {
    return Err(DateTimeParseError::TimezoneOverflow)
  }

  let mut tz = (hr * MINS_PER_HOUR + min) * SECS_PER_MINUTE + sec;

  if plus_or_minus == b'-' {
    tz = -tz;
  }

  if remains.is_some() {
    return Err(DateTimeParseError::BadFormat(
      format!("bad format in timezone: '{}'", tzstr)));
  }

  Ok(-tz)
}

/// datebsearch
/// Binary search -- from Knuth (6.2.1) Algorithm B.  Special case like this
/// is WAY faster than the generic bsearch().
pub fn datebsearch<'a>(key: &[u8], data: &'a [DateToken])
    -> Option<&'a DateToken> {

  let mut base = 0;
  let mut last = data.len() - 1;
  let mut position: usize;
  let mut result: i32;

  while last >= base {
    // get medium position
    position = base + ((last - base) >> 1);

    // precheck the first character for a bit of extra speed
    result = ((key[0] as i32) - data[position].token[0] as i32) as i32;
    if result == 0 {
      result = match key.cmp(data[position].token) {
        Ordering::Equal => return Some(&data[position]),
        Ordering::Less => -1,
        Ordering::Greater => 1,
      };
    }

    if result < 0 {
      last = position - 1;
    } else {
      base = position + 1;
    }
  }

  None
}

#[cfg(test)]
mod tests {
  use super::*;
  use super::DateTimeParseError::*;

  fn assert_decode_date(s: &str, year: i32, month: i32, day: i32, is2digity: bool) {
    let mut tmask: i32 = 0;
    let mut fmask: i32 = 0;
    let mut is2digits: bool = false;
    let mut tm = TimeMeta::empty();
    decode_date(s.as_bytes(), &mut tmask, &mut fmask, &mut is2digits, &mut tm).ok().unwrap();
    println!("{:?}", tm);
    assert_eq!(tm.tm_year, year);
    assert_eq!(tm.tm_mon, month);
    assert_eq!(tm.tm_mday, day);
    assert_eq!(is2digits, is2digity);
  }

  #[test]
  fn test_decode_date() {
    assert_decode_date("Feb-7-1997", 1997, 2, 7, false);
    assert_decode_date("2-7-1997", 1997, 2, 7, false);
    assert_decode_date("1997-2-7", 1997, 2, 7, false);
    //decode_date("Feb-7-1997".as_bytes(), &mut tmask, &mut fmask, &mut is2digits, &mut tm);
    //decode_date("2-7-1997".as_bytes(), &mut tmask, &mut fmask, &mut is2digits, &mut tm);
    //decode_date("1997-2-7".as_bytes(), &mut tmask, &mut fmask, &mut is2digits, &mut tm);
    //decode_date("1997.038".as_bytes(), &mut tmask, &mut fmask, &mut is2digits, &mut tm);
  }

  #[test]
  fn test_parse_fractional_second() {
    assert_eq!(12345000000i64, parse_fractional_second(".12345").ok().unwrap());
  }

  #[test]
  fn test_parse_fractional_second_fail1() {
    let err = parse_fractional_second(".inv").err().unwrap();
    assert_eq!(BadFormat("invalid digit found in string: '.inv'".to_owned()),
      err);
  }

  #[test]
  fn test_j2day() {
    let jd = date2j(2016, 11, 11);
    assert_eq!(5, j2day(jd));
  }

  #[test]
  fn test_decode_timezone() {
    assert_eq!(-3600, decode_timezone("+1").ok().unwrap());
    assert_eq!(3600,  decode_timezone("-1").ok().unwrap());
    assert_eq!(-5400, decode_timezone("+1:30").ok().unwrap());
    assert_eq!(5400,  decode_timezone("-1:30").ok().unwrap());
  }

  #[test]
  fn test_decode_timezone_failure() {
    match decode_timezone("+17") {
      Err(TimezoneOverflow) => {},
      _ => assert!(false, "Overflow must happen")
    };

    match decode_timezone("+1:60") {
      Err(TimezoneOverflow) => {},
      _ => assert!(false, "Overflow must happen")
    };

    match decode_timezone("+1:0:60") {
      Err(TimezoneOverflow) => {},
      _ => assert!(false, "Overflow must happen")
    };
  }

  #[test]
  fn test_datebsearch() {
    assert_eq!(b"april", datebsearch(b"april", &DATETK_TBL).unwrap().token);
    assert_eq!(b"monday", datebsearch(b"monday", &DATETK_TBL).unwrap().token);
    assert_eq!(b"friday", datebsearch(b"friday", &DATETK_TBL).unwrap().token);

    assert!(datebsearch(b"not_found", &DATETK_TBL).is_none());
  }
}