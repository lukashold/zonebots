// Native CPU wrapper predicates and actual static candidate/selection kernels.
// Wrapper external count/gun-capacity inputs and the unrelated1a9b0 refresh are
// explicit intercepted boundaries;16310/41a310/165d0/selection run unmodified.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
import java.nio.file.*;
public class ZNativeFactoryChoicesOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,F=0x72000000L,B=0x5c3f0cL;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void resume(EmulatorHelper e)throws Exception{long s=e.readRegister("ESP").longValue(),r=read(e,s,4);e.writeRegister("ESP",s+4);if(!e.run(toAddr(r),null,monitor))throw new Exception(e.getLastError());}
 void start(EmulatorHelper e,long entry)throws Exception{e.writeRegister("ESP",S);put(e,S,4,RET);e.writeRegister("ECX",F);if(!e.run(toAddr(entry),null,monitor))throw new Exception(e.getLastError());}
 public void run()throws Exception{
  byte[]robots=Files.readAllBytes(Paths.get("C:/GOG Games/Z/robots.dat")),weights=Files.readAllBytes(Paths.get("C:/GOG Games/Z/mult.dat"));
  try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
   long[]wrapper={0x416c60,0x4189e0,0x416ab0};long[]selectors={0x416970,0x416a10,0x4164a0,0x416530};
   for(int s=0;s<384;s++){
    EmulatorHelper e=new EmulatorHelper(currentProgram);try{
     e.writeMemory(toAddr(F),new byte[88]);e.writeMemory(toAddr(B),new byte[30]);put(e,F+0x20,4,B);put(e,F+6,1,s&8);put(e,F+6,1,(s&8)!=0?2:0);put(e,F+12,4,(s&16)!=0?2000:1000);put(e,B+25,4,new int[]{499,500,501,1000}[(s>>5)&3]);
     put(e,0x584972,1,(s&4)!=0?1:0);put(e,0x5c3ef8,4,Float.floatToRawIntBits((s&128)!=0?100:1));put(e,0x5c3f00,4,Float.floatToRawIntBits((s&128)!=0?100:0));
     for(long a:new long[]{0x4117f0,0x40f6f0,0x4106c0,0x416970,0x416a10,0x4164a0,0x416530})e.setBreakpoint(toAddr(a));start(e,wrapper[s%3]);
     long stop=e.readRegister("EIP").longValue();while(stop==0x4117f0||stop==0x40f6f0||stop==0x4106c0){e.writeRegister("EAX",stop==0x4117f0?(s&256)!=0?1:0:stop==0x40f6f0?((s/3)%5):((s/15)%2));resume(e);stop=e.readRegister("EIP").longValue();}
     out.println("{\"kind\":\"branch\",\"scenario\":"+s+",\"gun\":"+(stop==0x416530?1:0)+",\"column\":"+(stop==0x416530?e.readRegister("EDX").intValue():s%3==0?2:s%3==1?1:0)+",\"flags\":"+read(e,F+6,1)+"}");
    }finally{e.dispose();}
   }
   for(int s=0;s<128;s++){
    EmulatorHelper e=new EmulatorHelper(currentProgram);try{
     e.writeMemory(toAddr(0x615498),robots);e.writeMemory(toAddr(0x4a8980),weights);e.writeMemory(toAddr(F),new byte[88]);e.writeMemory(toAddr(B),new byte[64*30]);
     for(int i=0;i<64;i++)put(e,B+i*30,2,65535);put(e,B,2,0);put(e,B+10,2,2500-s*10);put(e,B+12,2,1+s*17);put(e,F+12,4,1000+(s/12)%5*1000);put(e,F+0x20,4,B);
     e.setBreakpoint(toAddr(RET));e.setBreakpoint(toAddr(0x41a9b0));e.writeRegister("EDX",(s/4)%3);start(e,selectors[s%4]);while(e.readRegister("EIP").longValue()!=RET){if(e.readRegister("EIP").longValue()!=0x41a9b0)throw new Exception("Unexpected selector stop");resume(e);}
     int count=(short)read(e,0x62dc38,2),h=0x811c9dc5;for(int i=0;i<count;i++)h=(h^(int)read(e,0x4a8384+i,1))*16777619;
     out.println("{\"kind\":\"selection\",\"scenario\":"+s+",\"product\":"+(e.readRegister("EAX").intValue()&255)+",\"count\":"+count+",\"candidates\":"+Integer.toUnsignedString(h)+"}");
    }finally{e.dispose();}
   }
  }
 }
}
