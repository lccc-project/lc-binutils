include "generic.td"


class XlangType {
    
}

class XLangScalarAttribute {}

class MinValue<int Value> : XlangScalarAttribute{
    int min = Value;
}

class MinValueArbitrary<list<bits<8>> Value> : XlangScalarAttribute{
    list<bits<8>> min = Value;
}

class MaxValue<int Value> : XlangScalarAttribute{
    int max = Value;
}

class MaxValueArbitrary<list<bits<8>> Value> : XlangScalarAttribute{
    list<bits<8>> max = Value;
}

def NonZero : XLangScalarAttribute{}

def Complex : XLangScalarAttribute{}

class VectorSize<int Size>: XLangScalarAttribute{
    int vector_size = Size;
}

class XLangScalar<int Bits,list<XLangScalarAttribute> Attributes> : XLangType{
    int scalar_bits = Bits;
    list<XLangScalarAttribute,list<XLangScalarAttribute> Attributes> attributes = Attributes
}

class XLangInt<int Bits,list<XLangScalarAttribute> Attributes> : XLangScalar<Bits,Attributes>{}

class XLangFloat<int Bits,list<XLangScalarAttribute> Attributes> : XLangScalar<Bits,Attributes>{}

class PointerAttribute{}

class Dereferenceable<int Size> : PointerAttribute{
    int deref_size = Size;
}

class DereferenceWrite<int Size> : PointerAttribute{
    int deref_size = Size;
}

class WriteOnly<int Size> : PointerAttribute{
    int deref_size = Size;
}

class NullDereferenceable<int Size> : PointerAttribute{
    int deref_size = Size;
}

class NullDereferenceWrite<int Size> : PointerAttribute{
    int deref_size = Size;
}

class NullWriteOnly<int Size> : PointerAttribute{
    int deref_size = Size;
}


class Aligned<int Size> : PointerAttribute{
    int alignment = Size;
}

def NonNull : PointerAttribute{}
def ReadOnly : PointerAttribute{}
def Unique : PointerAttribute{}
def Ref : PointerAttribute{}

class Pointer<XLangType Pointee,list<PointerAttribute> Attrs> : XLangType{
    XLangType type = Pointee;
    list<PointerAttributes> attrs = Attrs;
}


