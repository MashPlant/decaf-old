VTABLE(_Father) {
    <empty>
    Father
    _Father.foo;
}

VTABLE(_Child) {
    _Father
    Child
    _Father.foo;
}

VTABLE(_Main) {
    <empty>
    Main
}

FUNCTION(_Father_New) {
memo ''
_Father_New:
    _T1 = 8
    parm _T1
    _T2 =  call _Alloc
    _T3 = 0
    *(_T2 + 4) = _T3
    _T4 = VTBL <_Father>
    *(_T2 + 0) = _T4
    return _T2
}

FUNCTION(_Child_New) {
memo ''
_Child_New:
    _T5 = 12
    parm _T5
    _T6 =  call _Alloc
    _T7 = 0
    *(_T6 + 4) = _T7
    *(_T6 + 8) = _T7
    _T8 = VTBL <_Child>
    *(_T6 + 0) = _T8
    return _T6
}

FUNCTION(_Main_New) {
memo ''
_Main_New:
    _T9 = 4
    parm _T9
    _T10 =  call _Alloc
    _T11 = VTBL <_Main>
    *(_T10 + 0) = _T11
    return _T10
}

FUNCTION(_Father.foo) {
memo '_T0:4'
_Father.foo:
    _T12 = *(_T0 + 4)
    return _T12
}

FUNCTION(main) {
memo ''
main:
    _T15 = 3
    _T14 = _T15
    _T16 = 10
    _T17 = 0
    _T18 = (_T16 >= _T17)
    if (_T18 != 0) branch _L13
    _T19 = "Decaf runtime error: The length of the created array should not be less than 0.\n"
    parm _T19
    call _PrintString
    call _Halt
_L13:
    _T20 = 0
    _T21 = (_T16 < _T20)
    if (_T21 == 0) branch _L14
    _T22 = "Decaf runtime error: Cannot create negative-sized array\n"
    parm _T22
    call _PrintString
    call _Halt
_L14:
    _T23 = 4
    _T24 = (_T23 * _T16)
    _T25 = (_T23 + _T24)
    parm _T25
    _T26 =  call _Alloc
    *(_T26 + 0) = _T16
    _T27 = 0
    _T26 = (_T26 + _T25)
_L15:
    _T25 = (_T25 - _T23)
    if (_T25 == 0) branch _L16
    _T26 = (_T26 - _T23)
    *(_T26 + 0) = _T27
    branch _L15
_L16:
    _T28 = 0
_L17:
    _T29 = *(_T26 - 4)
    _T30 = (_T28 >= _T29)
    if (_T30 != 0) branch _L18
    _T31 = 4
    _T32 = (_T28 * _T31)
    _T33 = (_T26 + _T32)
    *(_T33 + 0) = _T14
    _T34 = 1
    _T35 = (_T28 + _T34)
    _T28 = _T35
    branch _L17
_L18:
    _T13 = _T26
    _T37 = 0
_L19:
    _T38 = *(_T13 - 4)
    _T39 = (_T37 >= _T38)
    if (_T39 != 0) branch _L20
    _T40 = 4
    _T41 = (_T37 * _T40)
    _T42 = (_T13 + _T41)
    _T43 = *(_T42 + 0)
    _T36 = _T43
    _T44 = 2
    _T45 = (_T14 > _T44)
    if (_T45 == 0) branch _L20
    _T46 = (_T14 + _T36)
    _T14 = _T46
    parm _T36
    call _PrintInt
    parm _T14
    call _PrintInt
    _T47 = "\n"
    parm _T47
    call _PrintString
    _T48 = 1
    _T49 = (_T37 + _T48)
    _T37 = _T49
    branch _L19
_L20:
}

