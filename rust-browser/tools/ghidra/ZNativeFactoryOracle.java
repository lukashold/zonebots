// Execute actual factory group creation41b730 and all real group/unit allocators.
// Only the final movement-command dispatcher45ef00 is intercepted and recorded.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
import java.nio.file.*;
public class ZNativeFactoryOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x5e30c8L,R=0x60ec00L,B=0x5c3f0cL,F=0x72000000L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void resume(EmulatorHelper e)throws Exception{long s=e.readRegister("ESP").longValue(),r=read(e,s,4);e.writeRegister("ESP",s+4);if(!e.run(toAddr(r),null,monitor))throw new Exception(e.getLastError());}
 public void run()throws Exception{
  byte[]robots=Files.readAllBytes(Paths.get("C:/GOG Games/Z/robots.dat"));
  try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
   for(int scenario=0;scenario<32;scenario++){
    EmulatorHelper e=new EmulatorHelper(currentProgram);try{
     e.writeMemory(toAddr(0x615498),robots);e.writeMemory(toAddr(U),new byte[240*192]);e.writeMemory(toAddr(R),new byte[240*72]);e.writeMemory(toAddr(F),new byte[88]);e.writeMemory(toAddr(B),new byte[64*30]);e.writeMemory(toAddr(0x586880),new byte[240]);
     for(int i=0;i<240;i++)for(int role=0;role<21;role++)put(e,R+i*72+19+role,1,255);for(int i=0;i<64;i++)put(e,B+i*30,2,65535);
     int owner=scenario%4,producer=(scenario/4)%4,product=producer==3?11:new int[]{2,6,17,18}[(scenario/16)%4];
     int dx=producer==0?(owner==1||owner==3?71:72):producer==3?23:32,dy=producer==0?(owner==1||owner==3?104:4):producer==3?32:50,exit=producer==0?(owner==1||owner==3?112:-48):producer==3?56:48;
     put(e,F,2,100+scenario*3);put(e,F+2,2,200+scenario*5);put(e,F+0x20,4,B);put(e,B+8,1,owner);put(e,0x478590,2,dx);put(e,0x47858c,2,dy);put(e,0x478408,2,exit);
     put(e,0x622cc0,4,0);put(e,0x622e18+owner,1,255);put(e,0x622dc2,1,99);put(e,0x584963,1,0);put(e,0x4a69f3,1,0);put(e,0x622dc4,1,0);put(e,0x622d1a,2,0);
     e.writeRegister("ESP",S);put(e,S,4,RET);put(e,S+4,4,owner);e.writeRegister("ECX",F);e.writeRegister("EDX",product);e.setBreakpoint(toAddr(RET));e.setBreakpoint(toAddr(0x45ef00));
     if(!e.run(toAddr(producer==3?0x40c340:0x40c180),null,monitor))throw new Exception(e.getLastError());
     while(e.readRegister("EIP").longValue()!=RET){
      if(e.readRegister("EIP").longValue()!=0x45ef00)throw new Exception("Unexpected factory stop");long unit=read(e,0x62dc04,4);int slot=(int)((unit-U)/192);
      out.println("{\"kind\":\"exit\",\"scenario\":"+scenario+",\"producer\":"+producer+",\"owner\":"+owner+",\"slot\":"+slot+",\"role\":"+(byte)read(e,unit+0x5e,1)+",\"x\":"+(short)read(e,unit+0x2c,2)+",\"y\":"+(short)read(e,unit+0x2e,2)+",\"goalX\":"+(short)read(e,0x62dc00,2)+",\"goalY\":"+(short)read(e,0x62dc3c,2)+"}");resume(e);
     }
     long group=read(e,0x622cd4,4);out.println("{\"kind\":\"group\",\"scenario\":"+scenario+",\"group\":"+((group-R)/72)+",\"count\":"+read(e,group+18,1)+",\"layout\":"+read(e,group+8,1)+",\"saved\":"+read(e,group+60,1)+"}");
    }finally{e.dispose();}
   }
  }
 }
}
