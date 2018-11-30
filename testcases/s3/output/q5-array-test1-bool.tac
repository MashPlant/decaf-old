VTABLE(_Main) {
    <empty>
    Main
}

FUNCTION(_Main_New) {
memo ''
_Main_New:
    _T0 = 4
    parm _T0
    _T1 =  call _Alloc
    _T2 = VTBL <_Main>
    *(_T1 + 0) = _T2
    return _T1
}

FUNCTION(main) {
memo ''
main:
    _T4 = 1
    _T5 = 3
    _T6 = 0
    _T7 = (_T5 >= _T6)
    if (_T7 != 0) branch _L10
    _T8 = "Decaf runtime error: The length of the created array should not be less than 0.\n"
    parm _T8
    call _PrintString
    call _Halt
_L10:
    _T9 = 0
    _T10 = (_T5 < _T9)
    if (_T10 == 0) branch _L11
    _T11 = "Decaf runtime error: Cannot create negative-sized array\n"
    parm _T11
    call _PrintString
    call _Halt
_L11:
    _T12 = 4
    _T13 = (_T12 * _T5)
    _T14 = (_T12 + _T13)
    parm _T14
    _T15 =  call _Alloc
    *(_T15 + 0) = _T5
    _T16 = 0
    _T15 = (_T15 + _T14)
_L12:
    _T14 = (_T14 - _T12)
    if (_T14 == 0) branch _L13
    _T15 = (_T15 - _T12)
    *(_T15 + 0) = _T16
    branch _L12
_L13:
    _T17 = 0
_L14:
    _T18 = *(_T15 - 4)
    _T19 = (_T17 >= _T18)
    if (_T19 != 0) branch _L15
    _T20 = 4
    _T21 = (_T17 * _T20)
    _T22 = (_T15 + _T21)
    *(_T22 + 0) = _T4
    _T23 = 1
    _T24 = (_T17 + _T23)
    _T17 = _T24
    branch _L14
_L15:
    _T3 = _T15
    _T25 = 1
    _T26 = *(_T3 - 4)
    _T27 = (_T25 < _T26)
    if (_T27 == 0) branch _L16
    _T28 = 0
    _T29 = (_T25 < _T28)
    if (_T29 == 0) branch _L17
_L16:
    _T30 = "Decaf runtime error: Array subscript out of bounds\n"
    parm _T30
    call _PrintString
    call _Halt
_L17:
    _T31 = 4
    _T32 = (_T25 * _T31)
    _T33 = (_T3 + _T32)
    _T34 = *(_T33 + 0)
    _T35 = 0
    _T36 = 4
    _T37 = (_T25 * _T36)
    _T38 = (_T3 + _T37)
    *(_T38 + 0) = _T35
    _T39 = 0
    _T40 = *(_T3 - 4)
    _T41 = (_T39 < _T40)
    if (_T41 == 0) branch _L18
    _T42 = 0
    _T43 = (_T39 < _T42)
    if (_T43 == 0) branch _L19
_L18:
    _T44 = "Decaf runtime error: Array subscript out of bounds\n"
    parm _T44
    call _PrintString
    call _Halt
_L19:
    _T45 = 4
    _T46 = (_T39 * _T45)
    _T47 = (_T3 + _T46)
    _T48 = *(_T47 + 0)
    parm _T48
    call _PrintBool
    _T49 = "\n"
    parm _T49
    call _PrintString
    _T50 = 1
    _T51 = *(_T3 - 4)
    _T52 = (_T50 < _T51)
    if (_T52 == 0) branch _L20
    _T53 = 0
    _T54 = (_T50 < _T53)
    if (_T54 == 0) branch _L21
_L20:
    _T55 = "Decaf runtime error: Array subscript out of bounds\n"
    parm _T55
    call _PrintString
    call _Halt
_L21:
    _T56 = 4
    _T57 = (_T50 * _T56)
    _T58 = (_T3 + _T57)
    _T59 = *(_T58 + 0)
    parm _T59
    call _PrintBool
    _T60 = "\n"
    parm _T60
    call _PrintString
}

