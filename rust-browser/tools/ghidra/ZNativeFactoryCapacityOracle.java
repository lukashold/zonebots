// Execute10570→10510 over actual240-unit/35-building pools;f580 uses an
// explicit10600 sector-count boundary, while its profile/signedness is native.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
import java.nio.file.*;
public class ZNativeFactoryCapacityOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,F=0x72000000L,B=0x5c3f0cL,U=0x5e30c8L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void call(EmulatorHelper e,long entry)throws Exception{e.writeRegister("ESP",S);put(e,S,4,RET);e.writeRegister("ECX",F);if(!e.run(toAddr(entry),null,monitor))throw new Exception(e.getLastError());}
 void resume(EmulatorHelper e)throws Exception{long s=e.readRegister("ESP").longValue(),r=read(e,s,4);e.writeRegister("ESP",s+4);if(!e.run(toAddr(r),null,monitor))throw new Exception(e.getLastError());}
 public void run()throws Exception{
  byte[]robots=Files.readAllBytes(Paths.get("C:/GOG Games/Z/robots.dat"));
  try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){for(int s=0;s<256;s++){
   EmulatorHelper e=new EmulatorHelper(currentProgram);try{
    e.writeMemory(toAddr(0x615498),robots);e.writeMemory(toAddr(F),new byte[88]);e.writeMemory(toAddr(B),new byte[30]);e.writeMemory(toAddr(0x5c5e38),new byte[35*88]);e.writeMemory(toAddr(U),new byte[240*192]);int team=s%4,queued=(s/4)%5,pending=(s&128)!=0?1:0,units=new int[]{-128,0,118,120}[(s/20)%4],limit=new int[]{0,120,127,255}[(s/80)%4],free=s%12;
    put(e,F+0x20,4,B);put(e,B+8,1,team);for(int i=0;i<35;i++)put(e,0x5c5e38+i*88+4,1,255);
    long build=0x5c5e38;put(e,build+4,1,0);put(e,build+30,2,100);put(e,build+32,4,B);for(int i=0;i<4;i++)put(e,build+82+i,1,i<queued?0:255);
    for(int i=0;i<5;i++)put(e,0x5ba0a4+i,1,255);put(e,0x5ba0a4+team,1,pending!=0?0:255);put(e,0x622ddd+team,1,units);put(e,0x5c47b4,2,limit);for(int i=0;i<240;i++)put(e,U+i*192+0x31,1,i<free?0:4);
    e.setBreakpoint(toAddr(RET));call(e,0x410570);int blocked=e.readRegister("EAX").intValue()&255;
    e.setBreakpoint(toAddr(0x410600));put(e,F+7,1,99);e.writeRegister("EDX",s%19);call(e,0x40f580);if(e.readRegister("EIP").longValue()==0x410600){e.writeRegister("EAX",s%7-1);resume(e);}int gun=e.readRegister("EAX").intValue()&255;
    out.println("{\"scenario\":"+s+",\"blocked\":"+blocked+",\"gun\":"+gun+",\"productRestored\":"+read(e,F+7,1)+"}");
   }finally{e.dispose();}
  }}
 }
}
