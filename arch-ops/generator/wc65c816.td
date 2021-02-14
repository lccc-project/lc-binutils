
include "generic.td"

class Wc65c816Flag<string Name>{
    string name = Name;
}

class Wc65c816StatusFlag<string Name,bits<8> Mask> : Wc65c816Flag<Name>{
    bits<8> mask = Mask;
}



def Carry     : Wc65c816StatusFlag<"c",0x01>{}
def Zero      : Wc65c816StatusFlag<"z",0x02>{}
def Interrupt : Wc65c816StatusFlag<"i",0x04>{}
def Decimal   : Wc65c816StatusFlag<"d",0x08>{}
def IndexWidth: Wc65c816StatusFlag<"x",0x10>{}
def Break     : Wc65c816StatusFlag<"b",0x10>{}
def MemWidth  : Wc65c816StatusFlag<"m",0x20>{}
def Overflow  : Wc65c816StatusFlag<"v",0x40>{}
def Minus     : Wc65c816StatusFlag<"m",0x80>{}
def Emulation : Wc65c816Flag<"e">{}


class Wc65c816Register<string Name,int Size> : Register<Name,Size>, Operand{

}

def ProcessorStatus : Wc65c816Register<"P",8>{
    list<Wc65c816Flag> flags = [Carry,Zero,Interrupt,Decimal,IndexWidth,MemWidth,Overflow,Minus];
}

class Wc65c816VariableRegister<string Name, Wc65c816Flag Flag> : Wc65c816Register<Name,16>{
    Wc65c816Flag sz_control = Flag;
}

def Accumulator   : Wc65c816VariableRegister<"A",MemWidth>  {}
def IndexX        : Wc65c816VariableRegister<"X",IndexWidth>{}
def IndexY        : Wc65c816VariableRegister<"Y",IndexWidth>{}

def DirectPage    : Wc65c816Register<"D",16> {}
def ProgramCounter: Wc65c816Register<"PC",16>{}
def StackPointer  : Wc65c816Register<"S",16> {}
def ProgramBank   : Wc65c816Register<"K",8>  {}
def DataBank      : Wc65c816Register<"B",8>  {}

class Wc65c816Instruction<string Name,bits<8> Opcode,list<Operand> Operands>: Instruction<Name,[Opcode],Operands>{}

class Wc65c816Scalar<Wc65c816Flag Control> : Operand{
    Wc65c816Flag control = Control;
}

class Wc65c816Address<int Size> : Operand{
    int size = Size;
}

def LongAddress : Wc65c816Address<24>{}
def ShortAddress : Wc65c816Address<16>{}
def Relative8 : Wc65c816Address<8>{}
def Relative16 : Wc65c816Address<16>{}
class Symbol<Wc65c816Address Underlying> : Wc65c816Address<Underlying.size>{
    Wc65c816Address underlying = Underlying;
}



def Nop : Wc65c816Instruction<"nop",0xea,[]>{}

def Wdm : Wc65c816Instruction<"wdm",0x42,[Optional<Scalar<8>>]>{}

def Rts : Wc65c816Instruction<"rts",0xb6,[]>{}
def Rtl : Wc65c816Instruction<"rtl",0x60,[]>{}

def Brk : Wc65c816Instruction<"brk",0x00,[Optional<Scalar<8>>]>{}
def Cop : Wc65c816Instruction<"cop",0x02,[Optional<Scalar<8>>]>{}

