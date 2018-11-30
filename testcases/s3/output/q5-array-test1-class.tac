VTABLE(_A) {
    <empty>
    A
    _A.seta;
    _A.printA;
}

VTABLE(_Main) {
    <empty>
    Main
}

FUNCTION(_A_New) {
memo ''
_A_New:
    _T3 = 8
    parm _T3
    _T4 =  call _Alloc
    _T5 = 0
    *(_T4 + 4) = _T5
    _T6 = VTBL <_A>
    *(_T4 + 0) = _T6
    return _T4
}

FUNCTION(_Main_New) {
memo ''
_Main_New:
    _T7 = 4
    parm _T7
    _T8 =  call _Alloc
    _T9 = VTBL <_Main>
    *(_T8 + 0) = _T9
    return _T8
}

FUNCTION(_A.seta) {
memo '_T0:4 _T1:8'
_A.seta:
    _T10 = *(_T0 + 4)
    *(_T0 + 4) = _T1
}

FUNCTION(_A.printA) {
memo '_T2:4'
_A.printA:
    _T11 = *(_T2 + 4)
    parm _T11
    call _PrintInt
    _T12 = "\n"
    parm _T12
    call _PrintString
}

FUNCTION(main) {
memo ''
main:
    _T15 =  call _A_New
    _T14 = _T15
    _T16 = 10
    parm _T14
    parm _T16
    _T17 = *(_T14 + 0)
    _T18 = *(_T17 + 8)
    call _T18
    _T19 = 6
    _T20 = 0
    _T21 = (_T19 >= _T20)
    if (_T21 != 0) branch _L13
    _T22 = "Decaf runtime error: The length of the created array should not be less than 0.\n"
    parm _T22
    call _PrintString
    call _Halt
_L13:
    _T23 = 0
    _T24 = (_T19 < _T23)
    if (_T24 == 0) branch _L14
    _T25 = "Decaf runtime error: Cannot create negative-sized array\n"
    parm _T25
    call _PrintString
    call _Halt
_L14:
    _T26 = 4
    _T27 = (_T26 * _T19)
    _T28 = (_T26 + _T27)
    parm _T28
    _T29 =  call _Alloc
    *(_T29 + 0) = _T19
    _T30 = 0
    _T29 = (_T29 + _T28)
_L15:
    _T28 = (_T28 - _T26)
    if (_T28 == 0) branch _L16
    _T29 = (_T29 - _T26)
    *(_T29 + 0) = _T30
    branch _L15
_L16:
    _T31 = 0
_L17:
    _T32 = *(_T29 - 4)
    _T33 = (_T31 >= _T32)
    if (_T33 != 0) branch _L18
    _T34 = 4
    _T35 = (_T31 * _T34)
    _T36 = (_T29 + _T35)
    _T37 =  call _A_New
    _T38 = *(_T14 + 4)
    *(_T37 + 4) = _T38
    *(_T36 + 0) = _T37
    _T39 = 1
    _T40 = (_T31 + _T39)
    _T31 = _T40
    branch _L17
_L18:
    _T13 = _T29
    _T41 = 1
    _T42 = *(_T13 - 4)
    _T43 = (_T41 < _T42)
    if (_T43 == 0) branch _L19
    _T44 = 0
    _T45 = (_T41 < _T44)
    if (_T45 == 0) branch _L20
_L19:
    _T46 = "Decaf runtime error: Array subscript out of bounds\n"
    parm _T46
    call _PrintString
    call _Halt
_L20:
    _T47 = 4
    _T48 = (_T41 * _T47)
    _T49 = (_T13 + _T48)
    _T50 = *(_T49 + 0)
    _T51 = 15
    parm _T50
    parm _T51
    _T52 = *(_T50 + 0)
    _T53 = *(_T52 + 8)
    call _T53
    _T54 = 0
    _T55 = *(_T13 - 4)
    _T56 = (_T54 < _T55)
    if (_T56 == 0) branch _L21
    _T57 = 0
    _T58 = (_T54 < _T57)
    if (_T58 == 0) branch _L22
_L21:
    _T59 = "Decaf runtime error: Array subscript out of bounds\n"
    parm _T59
    call _PrintString
    call _Halt
_L22:
    _T60 = 4
    _T61 = (_T54 * _T60)
    _T62 = (_T13 + _T61)
    _T63 = *(_T62 + 0)
    parm _T63
    _T64 = *(_T63 + 0)
    _T65 = *(_T64 + 12)
    call _T65
    _T66 = 1
    _T67 = *(_T13 - 4)
    _T68 = (_T66 < _T67)
    if (_T68 == 0) branch _L23
    _T69 = 0
    _T70 = (_T66 < _T69)
    if (_T70 == 0) branch _L24
_L23:
    _T71 = "Decaf runtime error: Array subscript out of bounds\n"
    parm _T71
    call _PrintString
    call _Halt
_L24:
    _T72 = 4
    _T73 = (_T66 * _T72)
    _T74 = (_T13 + _T73)
    _T75 = *(_T74 + 0)
    parm _T75
    _T76 = *(_T75 + 0)
    _T77 = *(_T76 + 12)
    call _T77
}

