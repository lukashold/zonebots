// Executes integer projectile updates and blast scheduling in the original x86
// emulator. Rendering, sound, and damage callbacks are observed or stubbed. No
// executable bytes or decompiled code are exported; output is numeric JSONL.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeProjectileOracle extends GhidraScript {
 static final long RET=0x12345000L,STACK=0x7fff0000L,U=0x1000000L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void set(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void resume(EmulatorHelper e,int pop)throws Exception{long s=e.readRegister("ESP").longValue(),r=read(e,s,4);e.writeRegister("ESP",s+4+pop);if(!e.run(toAddr(r),null,monitor))throw new Exception(e.getLastError());}
 void start(EmulatorHelper e,long entry,long ecx)throws Exception{
  e.writeRegister("ESP",STACK);set(e,STACK,4,RET);e.writeRegister("ECX",ecx);e.setBreakpoint(toAddr(RET));
  if(!e.run(toAddr(entry),null,monitor))throw new Exception(e.getLastError());
 }
 public void run()throws Exception{
  try(PrintWriter out=new PrintWriter(new FileOutputStream(getScriptArgs()[0]))){
   for(int rocket:new int[]{0,1})for(int[]delta:new int[][]{{60,22},{22,60},{-60,22},{-22,60},{60,-22},{22,-60},{-60,-22},{-22,-60},{160,0}}){
    EmulatorHelper e=new EmulatorHelper(currentProgram);
    try{
     e.writeMemory(toAddr(U),new byte[512]);e.writeMemory(toAddr(0x58aaaa),new byte[16384]);
     int dx=delta[0],dy=delta[1],major=Math.max(Math.abs(dx),Math.abs(dy)),minor=Math.min(Math.abs(dx),Math.abs(dy));
     set(e,U,2,500);set(e,U+2,2,500);set(e,U+4,2,major);set(e,U+6,2,major);set(e,U+8,2,major/2);set(e,U+10,2,minor);
     set(e,U+0x18,1,dy<0?255:1);set(e,U+0x19,1,Math.abs(dx)>=Math.abs(dy)?(dx>0?0:4):(dx>0?2:6));
     set(e,U+0x1a,1,rocket==1?10:8);set(e,U+0x1b,1,rocket==1?16:8);set(e,U+0x1d,1,50);
     set(e,U+0x1e,2,500);set(e,U+0x20,2,500);set(e,U+0x23,1,1);set(e,U+0x25,1,rocket==1?3:4);
     set(e,U+0x26,2,500+dx);set(e,U+0x28,2,500+dy);set(e,U+0x2d,1,100);
     int curve=Math.max(0,Math.min(4,(int)((float)Math.sqrt((dx*dx+dy*dy)>>7)-1)));
     int[][]curves={{16},{16,12,16},{16,12,8,12,16},{16,12,8,4,8,12,16},{16,12,8,4,0,4,8,12,16}};
     set(e,U+12,2,major/curves[curve].length);set(e,U+14,2,major/curves[curve].length);set(e,U+0x14,4,U+256);
     for(int i=0;i<curves[curve].length;i++)set(e,U+256+i,1,curves[curve][i]);
     for(long a:new long[]{0x462930,0x40bb50,0x463f60,0x466ac0,0x462f70,0x462fd0,0x462e80,0x420200,0x463910})e.setBreakpoint(toAddr(a));
     for(int tick=1;tick<=100;tick++){
      start(e,0x464000,U);boolean impact=false;
      for(int stops=0;e.readRegister("EIP").longValue()!=RET&&stops<40;stops++){
       long a=e.readRegister("EIP").longValue();
       if(a==0x420200||a==0x463910){impact=true;break;}
       resume(e,0);
      }
      out.println("{\"kind\":\"flight\",\"rocket\":"+rocket+",\"dx\":"+dx+",\"dy\":"+dy+",\"tick\":"+tick+",\"x\":"+read(e,0x635b64,2)+",\"y\":"+read(e,0x635b68,2)+",\"remaining\":"+read(e,U+4,2)+",\"phase\":"+read(e,U+16,4)+",\"acceleration\":"+(read(e,U+27,1)>>4)+",\"impact\":"+impact+"}");
      if(impact)break;
     }
    }finally{e.dispose();}
   }
   for(int initialPhase:new int[]{0,1}){
    EmulatorHelper e=new EmulatorHelper(currentProgram);
    try{
     for(int i=0;i<50;i++)set(e,0x595b12+i*10,2,65535);
     set(e,0x595b12,2,500);set(e,0x595b14,2,500);set(e,0x595b16,1,0);set(e,0x595b17,1,120);set(e,0x595b1a,1,0);
     set(e,0x498c24,4,0);set(e,0x498c20,4,0);e.setBreakpoint(toAddr(0x4208f0));
     for(int tick=1;tick<=24;tick++){
      set(e,0x622dc0,1,(tick+initialPhase)&1);start(e,0x420be0,0);boolean damage=e.readRegister("EIP").longValue()==0x4208f0;
      if(damage)resume(e,0);
      out.println("{\"kind\":\"blast\",\"initialPhase\":"+initialPhase+",\"tick\":"+tick+",\"frame\":"+read(e,0x595b16,1)+",\"active\":"+(read(e,0x595b12,2)!=65535)+",\"damage\":"+damage+"}");
     }
    }finally{e.dispose();}
   }
  }
 }
}
