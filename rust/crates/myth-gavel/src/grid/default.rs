//! Default 5×6 sentencing matrix per docs/04-CRATES/03-myth-gavel.md.

use myth_common::Enforcement;

pub const DEFAULT_MATRIX: [[Enforcement; 6]; 5] = [
    //       I                       II                      III                     IV                      V                       VI
    /* L1 */
    [
        Enforcement::Dismiss,
        Enforcement::Dismiss,
        Enforcement::Note,
        Enforcement::Note,
        Enforcement::Advisory,
        Enforcement::Advisory,
    ],
    /* L2 */
    [
        Enforcement::Note,
        Enforcement::Note,
        Enforcement::Advisory,
        Enforcement::Advisory,
        Enforcement::Caution,
        Enforcement::Caution,
    ],
    /* L3 */
    [
        Enforcement::Advisory,
        Enforcement::Caution,
        Enforcement::Caution,
        Enforcement::Warn,
        Enforcement::Warn,
        Enforcement::Warn,
    ],
    /* L4 */
    [
        Enforcement::Caution,
        Enforcement::Warn,
        Enforcement::Warn,
        Enforcement::Strike,
        Enforcement::Strike,
        Enforcement::Strike,
    ],
    /* L5 */
    [
        Enforcement::Strike,
        Enforcement::Strike,
        Enforcement::Strike,
        Enforcement::Strike,
        Enforcement::Strike,
        Enforcement::Strike,
    ],
];
