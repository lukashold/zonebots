// Actual5a080 protected-prefix instructions and54160→538f0 command preambles.
// Only actual path acceptance52810 is an external input (success); it does not
// affect the preceding native flags/timer/layout writes tested here.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeFactoryPrefixOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x5e30c8L,R=0x60ec00L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void call(EmulatorHelper e,long entry,long unit)throws Exception{e.writeRegister("ESP",S);put(e,S,4,RET);e.writeRegister("ECX",unit);if(!e.run(toAddr(entry),null,monitor))throw new Exception(e.getLastError());}
 void resume(EmulatorHelper e)throws Exception{long s=e.readRegister("ESP").longValue(),r=read(e,s,4);e.writeRegister("ESP",s+4);if(!e.run(toAddr(r),null,monitor))throw new Exception(e.getLastError());}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){for(int s=0;s<128;s++){
  EmulatorHelper e=new EmulatorHelper(currentProgram);try{
   e.writeMemory(toAddr(U),new byte[240*192]);e.writeMemory(toAddr(R),new byte[240*72]);for(int i=0;i<21;i++)put(e,R+19+i,1,i<2?i:255);
   boolean leader=(s&1)==0;long u=U+(leader?0:192);put(e,u+0x5f,1,0);put(e,u+0x5e,1,leader?255:1);put(e,u+0x9c,1,s*13);put(e,u+0xaa,1,new int[]{0,1,68,255}[(s>>1)&3]);put(e,u+0x5c,1,new int[]{-1,0,1,1}[(s>>3)&3]);put(e,u+0x7c,1,0);put(e,R+9,1,0xa5);put(e,R+8,1,255);put(e,R+60,1,2);put(e,R+16,1,s%3);put(e,0x632194,4,R);put(e,0x4a651e,1,0);put(e,0x4d02bb,1,0);
   e.writeRegister("ESI",u);e.setBreakpoint(toAddr(0x45a117));if(!e.run(toAddr(0x45a0e8),null,monitor))throw new Exception(e.getLastError());e.clearBreakpoint(toAddr(0x45a117));
   e.setBreakpoint(toAddr(RET));e.setBreakpoint(toAddr(0x453730));call(e,0x454160,u);boolean accepted=e.readRegister("EIP").longValue()==0x453730;e.clearBreakpoint(toAddr(0x453730));
   if(accepted){e.setBreakpoint(toAddr(0x452810));call(e,0x4538f0,u);if(e.readRegister("EIP").longValue()!=0x452810)throw new Exception("Expected path start boundary");e.writeRegister("EAX",0);resume(e);}
   out.println("{\"scenario\":"+s+",\"accepted\":"+(accepted?1:0)+",\"flags\":"+read(e,u+0x9c,1)+",\"protection\":"+read(e,u+0xaa,1)+",\"order\":"+(byte)read(e,u+0x5c,1)+",\"groupFlags\":"+read(e,R+9,1)+",\"layout\":"+read(e,R+8,1)+",\"bonus\":"+read(e,R+61,1)+"}");
  }finally{e.dispose();}
 }}}
}
