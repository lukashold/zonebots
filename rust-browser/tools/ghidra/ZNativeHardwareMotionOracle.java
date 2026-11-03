// Numeric-only native track/wheel counter004616f0. Cosmetic effects are
// intercepted; multiplayer bypasses the unrelated CRT cosmetic rand().
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeHardwareMotionOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x1000000L,W=U+0x1000;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(i*8);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void resume(EmulatorHelper e)throws Exception{long s=e.readRegister("ESP").longValue(),r=read(e,s,4);e.writeRegister("ESP",s+4);if(!e.run(toAddr(r),null,monitor))throw new Exception(e.getLastError());}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  for(int family:new int[]{6,25,30,24,32,34,36,38})for(int direction=0;direction<8;direction++)for(int start=0;start<4;start++)for(int freeze=0;freeze<3;freeze++){
   EmulatorHelper e=new EmulatorHelper(currentProgram);try{
    e.writeMemory(toAddr(U),new byte[0x2000]);put(e,U+8,4,W);put(e,W+8,2,300);put(e,W+10,2,300);put(e,U+0x30,1,family);put(e,U+0x39,1,direction);put(e,U+0x32,2,start);put(e,U+0x31,1,freeze==1?64:0);put(e,U+0x71,1,freeze==2?1:0);put(e,0x622dc4,1,1);
    for(long a:new long[]{RET,0x461100,0x460d30,0x460e20})e.setBreakpoint(toAddr(a));
    for(int tick=1;tick<=6;tick++){e.writeRegister("ESP",S);put(e,S,4,RET);e.writeRegister("ECX",U);if(!e.run(toAddr(0x4616f0),null,monitor))throw new Exception(e.getLastError());
     while(e.readRegister("EIP").longValue()!=RET)resume(e);
     out.println("{\"family\":"+family+",\"direction\":"+direction+",\"start\":"+start+",\"freeze\":"+freeze+",\"tick\":"+tick+",\"frame\":"+(short)read(e,U+0x32,2)+"}");
    }
   }finally{e.dispose();}
  }
 }}
}
