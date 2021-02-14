
class Register<string Name,int Size>{
    string name = Name;
    int size = Size;
}

class Operand{
}

class Instruction<string Name,list<bits<8>> Opcode,list<Operand> Operands>{
    string name = Name;
    list<bits<8>> opcode = Opcode;
    list<Operand> operands = Operands;
}

class Scalar<int Bits> : Operand{
    int size = Bits;
}

class Optional<Operand Of> : Operand{
    Operand of = Of;
}