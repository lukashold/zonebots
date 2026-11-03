// Actual52810, with525a0 line initialization isolated and its arguments recorded.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativePathStartOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x5e30c8L,P=0x5eee28L,Q=P+8;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void resume(EmulatorHelper e)throws Exception{long s=e.readRegister("ESP").longValue(),r=read(e,s,4);e.writeRegister("ESP",s+4);if(!e.run(toAddr(r),null,monitor))throw new Exception(e.getLastError());}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){for(int s=0;s<512;s++){
  EmulatorHelper e=new EmulatorHelper(currentProgram);try{e.writeMemory(toAddr(U),new byte[192]);put(e,U+0x31,1,s*5);put(e,U+0x9c,1,s*7);put(e,U+0x64,1,s*3);put(e,U+0x71,1,s);put(e,U+0x8c,1,s*11);put(e,U+0x61,1,s*13);put(e,U+0x14,4,s%3!=0?P:0);put(e,U+0xc,4,s%5!=0?Q:0);put(e,P,2,-300+s*17);put(e,P+2,2,400-s*23);put(e,Q,2,200+s*7);put(e,Q+2,2,-250-s*13);
   e.setBreakpoint(toAddr(RET));e.setBreakpoint(toAddr(0x4525a0));e.writeRegister("ESP",S);put(e,S,4,RET);e.writeRegister("ECX",U);if(!e.run(toAddr(0x452810),null,monitor))throw new Exception(e.getLastError());int called=0,x=0,y=0;if(e.readRegister("EIP").longValue()==0x4525a0){called=1;x=(short)e.readRegister("EDX").longValue();y=(short)read(e,e.readRegister("ESP").longValue()+4,2);e.writeRegister("EAX",0);resume(e);}
   out.println("{\"scenario\":"+s+",\"called\":"+called+",\"x\":"+x+",\"y\":"+y+",\"flags64\":"+read(e,U+0x64,1)+",\"flags71\":"+read(e,U+0x71,1)+",\"contact8c\":"+read(e,U+0x8c,1)+",\"phase61\":"+read(e,U+0x61,1)+"}");
  }finally{e.dispose();}
 }}}
}
