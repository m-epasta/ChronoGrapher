use chronographer::prelude::*;

calendar! {
   bsecond: 3
}; // Unexpected field "bsecond", did you mean "second"? 
calendar! {
   second: 
}; // Expected calendar field literal but got nothing
calendar! {
   second 
}; // Expected ":" after name
calendar! {
   minute: 5
   second: 0
}; // Expected a semicolon ";" but got "second: 0"
calendar! {
   minute: 500
}; // Value exceeds its expected range, expected it to be in 0..=59 minutes range
calendar! {
   minute: +500
}; // Value exceeds its expected range, expected it to be in 0..=59 minutes range
calendar! {
   minute: -1
}; // Negative numbers are not allowed as a calendar field literal
calendar! {
   minute: *..=10
}; // Expected positive number for range but got "*" instead
calendar! {
   minute: 1;
   minute: 2;
}; // The "minute" calendar field has already been defined before
calendar! {
   minute: 1.5
}; // Decimal numbers are not allowed in a calendar field literal
calendar! {
   minute: 1,2,3,
}; // Expected a number after trialing comma but found nothing
calendar! {
   minute: 1,2,3,;
   second: 10
}; // Expected a number after trialing comma but found ";"
calendar! {
   minute: 10;
   hour: 1,2,3;
}; // Incorrect ordering, expected after "minute" either "second" or "millisecond" but got "hour" instead
calendar! {
   hour: 1,2,1,3;
}; // Repeated item found in list, expected unique entries
calendar! {
   hour: first(2);
   minute: +10;
}; // The ``first(...)`` literal is only allowed in the ``day`` calendar field and not ``hour``
calendar! {
   millisecond: last(2);
}; // The ``last(...)`` literal is only allowed in the ``day`` calendar field and not ``millisecond``
calendar! {
   day: last(+3);
}; // Expected a number, a range or a list for ``last(...)`` but got ``+3``
calendar! {
   hour: MON;
}; // The ``MON`` constant is only applicable to the ``day`` calendar field and not ``hour`` calendar field.
calendar! {
   millisecond: APR;
}; // The ``APR` constant is only applicable to the ``month`` calendar field and not ``milliseecond`` calendar field.
calendar! {
   millisecond: 4, 2, 3;
}; // Expected a larger literal to follow after the number "4" but got the number "2"
calendar! {
   millisecond: 4..6, 2, 3;
}; // Expected a larger literal to follow after the range "4..6" but got the number "2"
