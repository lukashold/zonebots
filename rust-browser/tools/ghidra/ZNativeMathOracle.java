// Read-only numeric behavior oracle for the locally owned GOG executable.
// Run against an existing Ghidra12.2 project with -readOnly -noanalysis.
// Emulator writes stay in emulated memory; no program or resources are exported.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
import java.math.BigInteger;
public class ZNativeMathOracle extends GhidraScript {
 private static final long RET=0x12345000L, STACK=0x7fff0000L;
 private void call(EmulatorHelper e,long entry) throws Exception {
  e.writeRegister("ESP",STACK);e.writeMemoryValue(toAddr(STACK),4,RET);e.setBreakpoint(toAddr(RET));
  if(!e.run(toAddr(entry),null,monitor))throw new Exception(e.getLastError());
 }
 private long word(EmulatorHelper e,long a){byte[] b=e.readMemory(toAddr(a),4);return ((b[0]&255L)|((b[1]&255L)<<8)|((b[2]&255L)<<16)|((b[3]&255L)<<24));}
 public void run() throws Exception {
  try(PrintWriter out=new PrintWriter(new FileOutputStream(getScriptArgs()[0]))){
   for(long seed:new long[]{0,1,371,7291,0xffffffffL}){
    EmulatorHelper e=new EmulatorHelper(currentProgram);
    try{
     e.writeRegister("ECX",seed);call(e,0x460850L);
     out.print("{\"kind\":\"rng\",\"seed\":"+seed+",\"state\":[");
     for(int i=0;i<5;i++){if(i>0)out.print(",");out.print(word(e,0x622cdcL+4*i));}
     out.print("],\"values\":[");for(int i=0;i<10;i++){call(e,0x460910L);if(i>0)out.print(",");out.print(e.readRegister("EAX").longValue()&0xffffffffL);}out.println("]}");
    }finally{e.dispose();}
   }
   EmulatorHelper e=new EmulatorHelper(currentProgram);
   try {
    for(int dx=-13;dx<=13;dx++)for(int dy=-13;dy<=13;dy++){
     if(dx==0&&dy==0)continue;e.writeRegister("ECX",0);e.writeRegister("EDX",0);
     e.writeMemoryValue(toAddr(STACK+4),4,dx);e.writeMemoryValue(toAddr(STACK+8),4,dy);call(e,0x459af0L);
     out.println("{\"kind\":\"direction\",\"dx\":"+dx+",\"dy\":"+dy+",\"result\":"+(e.readRegister("EAX").intValue()&255)+"}");
    }
   }finally{e.dispose();}
   for(int[] delta:new int[][]{{60,22},{22,60},{-60,22},{-22,60},{60,-22},{22,-60},{-60,-22},{-22,-60}}){
    e=new EmulatorHelper(currentProgram);
    try{
     int dx=delta[0],dy=delta[1],major=Math.max(Math.abs(dx),Math.abs(dy)),minor=Math.min(Math.abs(dx),Math.abs(dy));long unit=0x1000000L;
     e.writeMemoryValue(toAddr(unit+0x10),2,major);e.writeMemoryValue(toAddr(unit+0x12),2,major);e.writeMemoryValue(toAddr(unit+0x14),2,major/2);e.writeMemoryValue(toAddr(unit+0x16),2,minor);e.writeMemoryValue(toAddr(unit+0xc),1,15);e.writeMemoryValue(toAddr(unit+0xe),1,dy<0?255:1);
     e.writeMemoryValue(toAddr(0x635b64L),2,500);e.writeMemoryValue(toAddr(0x635b68L),2,500);
     long entry=Math.abs(dx)>=Math.abs(dy)?(dx>0?0x463200L:0x463350L):(dx>0?0x4632b0L:0x463400L);
     out.print("{\"kind\":\"ray\",\"dx\":"+dx+",\"dy\":"+dy+",\"positions\":[");
     for(int t=0;t<4;t++){
      e.writeRegister("ECX",unit);call(e,entry);if(t>0)out.print(",");
      byte[] x=e.readMemory(toAddr(0x635b64L),2),y=e.readMemory(toAddr(0x635b68L),2);
      out.print("["+((x[0]&255)|((x[1]&255)<<8))+","+((y[0]&255)|((y[1]&255)<<8))+"]");
     }out.println("]}");
    }finally{e.dispose();}
   }
  }
 }
}
