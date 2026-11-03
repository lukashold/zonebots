// Native defense cleanup14570, selection15950 and captured-sector14720.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeDefenseTaskOracle extends GhidraScript{
 static final long RET=0x12345000L,STACK=0x7fff0000L,SECT=0x5c3f0cL,GROUP=0x5c47b8L,UNIT=0x5e30c8L,ROSTER=0x60ec00L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void resume(EmulatorHelper e)throws Exception{long s=e.readRegister("ESP").longValue(),r=read(e,s,4);e.writeRegister("ESP",s+4);if(!e.run(toAddr(r),null,monitor))throw new Exception(e.getLastError());}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  for(int phase=0;phase<3;phase++)for(int scenario=0;scenario<16;scenario++){
   EmulatorHelper e=new EmulatorHelper(currentProgram);
   try{
    e.writeMemory(toAddr(GROUP),new byte[240*24]);e.writeMemory(toAddr(ROSTER),new byte[240*72]);e.writeMemory(toAddr(UNIT),new byte[240*192]);
    for(int i=0;i<64;i++){e.writeMemory(toAddr(SECT+i*30),new byte[30]);put(e,SECT+i*30,2,65535);}
    for(int i=0;i<2;i++){long s=SECT+i*30;put(e,s,2,i*8);put(e,s+2,2,0);put(e,s+4,2,i*8+8);put(e,s+6,2,8);put(e,s+8,1,phase==0&&(scenario&8)!=0&&i==0?0:1);put(e,s+18,1,4+i*8);put(e,s+19,1,4);put(e,s+24,1,i==0&&(scenario&8)!=0?2:0);put(e,s+25,4,i==0?300+scenario*180:1500);}
    put(e,0x5c3f00,4,Float.floatToRawIntBits((scenario%4)*100f));put(e,0x5c3ef8,4,Float.floatToRawIntBits(300f));put(e,0x4a6813,1,0);
    for(int i=0;i<5;i++){
     long g=GROUP+i*24,u=UNIT+i*192;put(e,g,4,SECT+(phase==2?0:30));put(e,g+4,4,phase==0?SECT+(i%2)*30:(phase==1&&i==3&&(scenario&4)!=0?SECT+30:-1));put(e,g+8,4,i==4?50:200+i*100);put(e,g+12,4,-1);put(e,g+16,1,24);put(e,g+17,1,phase==0?4:phase==1&&i==3&&(scenario&4)!=0?4:255);put(e,g+21,1,phase==2&&i==4?0:1);
     put(e,ROSTER+i*72+19,1,i);put(e,u+0x31,1,5);put(e,u+0x2c,2,32+i*16);put(e,u+0x2e,2,64);put(e,u+0x7c,1,i==4?2:0);put(e,u+0x30,1,4);
    }
    for(long a:new long[]{RET,0x419430})e.setBreakpoint(toAddr(a));
    e.writeRegister("ESP",STACK);put(e,STACK,4,RET);e.writeRegister("ECX",SECT);
    if(!e.run(toAddr(phase==0?0x414570:phase==1?0x415950:0x414720),null,monitor))throw new Exception(e.getLastError());int order=0;
    while(e.readRegister("EIP").longValue()!=RET){long a=e.readRegister("EIP").longValue(),sp=e.readRegister("ESP").longValue();
     if(a==0x419430){int group=(int)((e.readRegister("EDX").longValue()-UNIT)/192);int sector=(int)((read(e,sp+4,4)-SECT)/30);out.println("{\"kind\":\"order\",\"phase\":"+phase+",\"scenario\":"+scenario+",\"order\":"+order+++",\"group\":"+group+",\"sector\":"+sector+"}");}
     else throw new Exception("Unexpected defense stop");resume(e);
    }
    for(int i=0;i<5;i++){long g=GROUP+i*24;long via=read(e,g+4,4),assigned=read(e,g+12,4);out.println("{\"kind\":\"group\",\"phase\":"+phase+",\"scenario\":"+scenario+",\"group\":"+i+",\"task\":"+(byte)read(e,g+17,1)+",\"via\":"+(via==0xffffffffL?-1:(via-SECT)/30)+",\"assigned\":"+(assigned==0xffffffffL?-1:(assigned-SECT)/30)+"}");}
   }finally{e.dispose();}
  }
 }}
}
