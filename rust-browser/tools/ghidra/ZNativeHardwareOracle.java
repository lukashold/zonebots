// Numeric x86 traces for turret turns, hatch transitions, and hatch artwork.
// All output is behavior/geometry; the local program database stays read-only.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeHardwareOracle extends GhidraScript {
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x1000000L,W=U+0x1000,G=U+0x2000,O=U+0x3000;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void invoke(EmulatorHelper e,long entry)throws Exception{e.writeRegister("ESP",S);put(e,S,4,RET);e.writeRegister("ECX",U);e.setBreakpoint(toAddr(RET));if(!e.run(toAddr(entry),null,monitor))throw new Exception(e.getLastError());}
 void resume(EmulatorHelper e)throws Exception{long s=e.readRegister("ESP").longValue(),r=read(e,s,4);e.writeRegister("ESP",s+4);if(!e.run(toAddr(r),null,monitor))throw new Exception(e.getLastError());}
 void init(EmulatorHelper e)throws Exception{e.writeMemory(toAddr(U),new byte[0x4000]);put(e,U+8,4,W);put(e,0x632194,4,G);put(e,U+0x2c,2,500);put(e,U+0x2e,2,500);put(e,U+0x30,1,6);put(e,U+0x7c,1,1);put(e,U+4,1,30);put(e,W+5,1,20);put(e,W+0xd,1,15);put(e,W+0x24,1,80);}
 public void run()throws Exception{
  try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
   for(int direction=0;direction<8;direction++){
    EmulatorHelper e=new EmulatorHelper(currentProgram);
    try{
     init(e);put(e,U+0x3c,1,15);put(e,W+0x20,1,80);e.setBreakpoint(toAddr(0x449470));e.setBreakpoint(toAddr(0x45ae80));
     int[][]delta={{0,-100},{100,-100},{100,0},{100,100},{0,100},{-100,100},{-100,0},{-100,-100}};
     for(int tick=1;tick<=48;tick++){
      put(e,0x622dc0,1,tick&1);int reload=(int)read(e,U+0x7b,1);if(reload>0)put(e,U+0x7b,1,reload-1);
      invoke(e,0x45b050);boolean shot=false;
      while(e.readRegister("EIP").longValue()!=RET){long a=e.readRegister("EIP").longValue(),s=e.readRegister("ESP").longValue();
       if(a==0x449470){put(e,read(e,s+8,4),2,514+delta[direction][0]);put(e,read(e,s+12,4),2,512+delta[direction][1]);e.writeRegister("EAX",0);}
       else if(a==0x45ae80){shot=true;e.writeRegister("EAX",0);}else throw new Exception("Unexpected turret stop");resume(e);
      }
      out.println("{\"kind\":\"turret\",\"direction\":"+direction+",\"tick\":"+tick+",\"aim\":"+read(e,W+0x15,1)+",\"timer\":"+(byte)read(e,U+0x3c,1)+",\"shot\":"+shot+"}");
     }
    }finally{e.dispose();}
   }
   for(int scenario=0;scenario<4;scenario++){
    EmulatorHelper e=new EmulatorHelper(currentProgram);
    try{
     init(e);put(e,W+0x19,1,scenario==2?3:255);put(e,W+0x1a,1,scenario==2?255:1);put(e,W+0x1b,1,scenario==2?25:5);e.setBreakpoint(toAddr(0x458500));e.setBreakpoint(toAddr(0x4160c0));e.setBreakpoint(toAddr(0x45b210));
     for(int tick=1;tick<=64;tick++){
      invoke(e,0x45b610);boolean shot=false;
      while(e.readRegister("EIP").longValue()!=RET){long a=e.readRegister("EIP").longValue();if(a==0x458500)e.writeRegister("EAX",scenario==1?0xff00:0x0103);else if(a==0x45b210)shot=true;else if(a!=0x4160c0)throw new Exception("Unexpected hatch stop");resume(e);}
      out.println("{\"kind\":\"hatch\",\"scenario\":"+scenario+",\"tick\":"+tick+",\"frame\":"+(byte)read(e,W+0x19,1)+",\"motion\":"+(byte)read(e,W+0x1a,1)+",\"timer\":"+(byte)read(e,W+0x1b,1)+",\"shot\":"+shot+"}");
     }
    }finally{e.dispose();}
   }
   for(int family:new int[]{6,25,30})for(int direction=0;direction<8;direction++)for(int frame=0;frame<5;frame++){
    EmulatorHelper e=new EmulatorHelper(currentProgram);
    try{init(e);put(e,U+0x30,1,family);put(e,W+0x15,1,direction);put(e,W+0x19,1,frame);put(e,0x637c80,2,100);put(e,0x637c84,2,200);put(e,S+4,4,O);e.writeRegister("EDX",U);e.writeRegister("ECX",W);e.writeRegister("ESP",S);put(e,S,4,RET);e.setBreakpoint(toAddr(RET));if(!e.run(toAddr(0x454e30),null,monitor))throw new Exception(e.getLastError());out.println("{\"kind\":\"crewSprite\",\"family\":"+family+",\"direction\":"+direction+",\"frame\":"+frame+",\"x\":"+read(e,O,2)+",\"y\":"+read(e,O+2,2)+",\"sprite\":"+read(e,O+4,2)+"}");}finally{e.dispose();}
   }
  }
 }
}
