// Actual40a8a0 and40b2a0 work/completion branch. Capacity is an explicit gate
// input at40c080; group/weapon duration41a310 executes unmodified on GOG data.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
import java.nio.file.*;
public class ZNativeFactoryWorkOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,F=0x72000000L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void resume(EmulatorHelper e)throws Exception{long s=e.readRegister("ESP").longValue(),r=read(e,s,4);e.writeRegister("ESP",s+4);if(!e.run(toAddr(r),null,monitor))throw new Exception(e.getLastError());}
 public void run()throws Exception{
  byte[]robots=Files.readAllBytes(Paths.get("C:/GOG Games/Z/robots.dat")),weights=Files.readAllBytes(Paths.get("C:/GOG Games/Z/mult.dat"));
  try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){for(int s=0;s<256;s++){
   EmulatorHelper e=new EmulatorHelper(currentProgram);try{
    e.writeMemory(toAddr(0x615498),robots);e.writeMemory(toAddr(0x4a8980),weights);e.writeMemory(toAddr(F),new byte[88]);int product=s%19,divisor=new int[]{1,12,17,221}[(s/19)%4];put(e,F+7,1,product);put(e,F+44,4,s%3==0?-1234:s%3==1?500:100000);put(e,0x622dba,2,divisor);
    e.setBreakpoint(toAddr(RET));e.writeRegister("ESP",S);put(e,S,4,RET);e.writeRegister("ECX",F);if(!e.run(toAddr(0x40a8a0),null,monitor))throw new Exception(e.getLastError());
    out.println("{\"kind\":\"initialize\",\"scenario\":"+s+",\"remaining\":"+(int)read(e,F+36,4)+",\"total\":"+(int)read(e,F+40,4)+",\"animation\":"+(short)read(e,F+10,2)+"}");
    int rate=s%36,remaining=new int[]{-1,0,1,rate,rate-1,rate+1,50000}[s%7],animation=new int[]{0,-27,17,-32768,32767}[s%5];put(e,F+36,4,remaining);put(e,F+10,2,animation);put(e,F+54,2,0);
    e.clearBreakpoint(toAddr(RET));for(long a:new long[]{0x40c080,0x40b4ce,0x40b413})e.setBreakpoint(toAddr(a));e.writeRegister("ESI",F);e.writeRegister("EAX",rate);e.writeRegister("ESP",S);if(!e.run(toAddr(0x40b3bb),null,monitor))throw new Exception(e.getLastError());
    if(e.readRegister("EIP").longValue()==0x40c080){e.writeRegister("EAX",(s&1)!=0?1:0);resume(e);}long stop=e.readRegister("EIP").longValue();if(stop!=0x40b4ce&&stop!=0x40b413)throw new Exception("Unexpected work stop");
    out.println("{\"kind\":\"work\",\"scenario\":"+s+",\"completed\":"+(stop==0x40b413?1:0)+",\"remaining\":"+(int)read(e,F+36,4)+",\"total\":"+(int)read(e,F+40,4)+",\"animation\":"+(short)read(e,F+10,2)+",\"hold\":"+(short)read(e,F+54,2)+"}");
   }finally{e.dispose();}
  }}
 }
}
