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
    _T6 = 2
    _T4 = _T6
    _T7 = 2
    _T8 = 3
    _T9 = (_T7 - _T8)
    _T5 = _T9
    _T10 = 3
    _T11 = 0
    _T12 = (_T5 >= _T11)
    if (_T12 != 0) branch _L10
    _T13 = "Decaf runtime error: The length of the created array should not be less than 0.\n"
    parm _T13
    call _PrintString
    call _Halt
_L10:
    _T14 = 0
    _T15 = (_T5 < _T14)
    if (_T15 == 0) branch _L11
    _T16 = "Decaf runtime error: Cannot create negative-sized array\n"
    parm _T16
    call _PrintString
    call _Halt
_L11:
    _T17 = 4
    _T18 = (_T17 * _T5)
    _T19 = (_T17 + _T18)
    parm _T19
    _T20 =  call _Alloc
    *(_T20 + 0) = _T5
    _T21 = 0
    _T20 = (_T20 + _T19)
_L12:
    _T19 = (_T19 - _T17)
    if (_T19 == 0) branch _L13
    _T20 = (_T20 - _T17)
    *(_T20 + 0) = _T21
    branch _L12
_L13:
    _T22 = 0
_L14:
    _T23 = *(_T20 - 4)
    _T24 = (_T22 >= _T23)
    if (_T24 != 0) branch _L15
    _T25 = 4
    _T26 = (_T22 * _T25)
    _T27 = (_T20 + _T26)
    *(_T27 + 0) = _T10
    _T28 = 1
    _T29 = (_T22 + _T28)
    _T22 = _T29
    branch _L14
_L15:
    _T3 = _T20
    _T30 = 1
    _T31 = *(_T3 - 4)
    _T32 = (_T30 < _T31)
    if (_T32 == 0) branch _L16
    _T33 = 0
    _T34 = (_T30 < _T33)
    if (_T34 == 0) branch _L17
_L16:
    _T35 = "Decaf runtime error: Array subscript out of bounds\n"
    parm _T35
    call _PrintString
    call _Halt
_L17:
    _T36 = 4
    _T37 = (_T30 * _T36)
    _T38 = (_T3 + _T37)
    _T39 = *(_T38 + 0)
    _T40 = 1
    _T41 = *(_T3 - 4)
    _T42 = (_T40 < _T41)
    if (_T42 == 0) branch _L18
    _T43 = 0
    _T44 = (_T40 < _T43)
    if (_T44 == 0) branch _L19
_L18:
    _T45 = "Decaf runtime error: Array subscript out of bounds\n"
    parm _T45
    call _PrintString
    call _Halt
_L19:
    _T46 = 4
    _T47 = (_T40 * _T46)
    _T48 = (_T3 + _T47)
    _T49 = *(_T48 + 0)
    _T50 = 1
    _T51 = (_T49 + _T50)
    _T52 = 4
    _T53 = (_T30 * _T52)
    _T54 = (_T3 + _T53)
    *(_T54 + 0) = _T51
    _T55 = 0
    _T56 = *(_T3 - 4)
    _T57 = (_T55 < _T56)
    if (_T57 == 0) branch _L20
    _T58 = 0
    _T59 = (_T55 < _T58)
    if (_T59 == 0) branch _L21
_L20:
    _T60 = "Decaf runtime error: Array subscript out of bounds\n"
    parm _T60
    call _PrintString
    call _Halt
_L21:
    _T61 = 4
    _T62 = (_T55 * _T61)
    _T63 = (_T3 + _T62)
    _T64 = *(_T63 + 0)
    parm _T64
    call _PrintInt
    _T65 = "\n"
    parm _T65
    call _PrintString
    _T66 = 1
    _T67 = *(_T3 - 4)
    _T68 = (_T66 < _T67)
    if (_T68 == 0) branch _L22
    _T69 = 0
    _T70 = (_T66 < _T69)
    if (_T70 == 0) branch _L23
_L22:
    _T71 = "Decaf runtime error: Array subscript out of bounds\n"
    parm _T71
    call _PrintString
    call _Halt
_L23:
    _T72 = 4
    _T73 = (_T66 * _T72)
    _T74 = (_T3 + _T73)
    _T75 = *(_T74 + 0)
    parm _T75
    call _PrintInt
    _T76 = "\n"
    parm _T76
    call _PrintString
}

