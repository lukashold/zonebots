// Numeric-only execution of follower mask generation and walking counters.
// No file in the original installation or Ghidra database is modified.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeAnimationOracle extends GhidraScript {
 static final long RET=0x12345000L,STACK=0x7fff0000L,U=0x1000000L;
 int read(EmulatorHelper e,long a){return e.readMemory(toAddr(a),1)[0]&255;}
 void set(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void invoke(EmulatorHelper e,long entry)throws Exception{e.writeRegister("ESP",STACK);set(e,STACK,4,RET);e.setBreakpoint(toAddr(RET));if(!e.run(toAddr(entry),null,monitor))throw new Exception(e.getLastError());}
 public void run()throws Exception{
  try(PrintWriter out=new PrintWriter(new FileOutputStream(getScriptArgs()[0]))){
   for(int speed=0;speed<13;speed++){
    EmulatorHelper e=new EmulatorHelper(currentProgram);
    try{
     e.writeMemory(toAddr(U),new byte[192]);set(e,U+0x35,1,2);set(e,U+0x5d,1,1);set(e,U+0x81,1,1);set(e,U+0x8b,1,speed);
     set(e,U+0x3c,1,read(e,0x49a6e4+speed));set(e,STACK+4,4,U);
     for(int tick=1;tick<=48;tick++){
      set(e,STACK+4,4,U);e.writeRegister("ECX",0);invoke(e,0x452200);
      out.println("{\"kind\":\"walk\",\"speed\":"+speed+",\"tick\":"+tick+",\"frame\":"+read(e,U+0x32)+",\"timer\":"+(byte)read(e,U+0x3c)+"}");
     }
    }finally{e.dispose();}
   }
   long[]masks={0x49a218,0x49a600,0x49a308,0x49a500,0x49a400};
   for(int layout=0;layout<masks.length;layout++){
    EmulatorHelper e=new EmulatorHelper(currentProgram);
    try{
     e.writeMemory(toAddr(U),new byte[1025]);set(e,U+1024,1,4);set(e,STACK+4,4,U+1024);
     e.writeRegister("ECX",masks[layout]);e.writeRegister("EDX",U);invoke(e,0x44b6d0);
     for(int slot=0;slot<20;slot++)out.println("{\"kind\":\"formation\",\"layout\":"+layout+",\"slot\":"+(slot+1)+",\"x\":"+(byte)read(e,U+slot*2)+",\"y\":"+(byte)read(e,U+slot*2+1)+"}");
    }finally{e.dispose();}
   }
  }
 }
}
