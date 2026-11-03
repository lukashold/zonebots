// Execute native candidate collection, ordering, and initial sector allocation.
// Final move dispatch is intercepted; no instruction bytes are exported.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeTaskOracle extends GhidraScript{
 static final long RET=0x12345000L,STACK=0x7fff0000L,SECT=0x5c3f0cL,GROUP=0x5c47b8L,UNIT=0x5e30c8L,ROSTER=0x60ec00L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void invoke(EmulatorHelper e,long a)throws Exception{e.writeRegister("ESP",STACK);put(e,STACK,4,RET);e.writeRegister("ECX",SECT);if(!e.run(toAddr(a),null,monitor))throw new Exception(e.getLastError());}
 void resume(EmulatorHelper e)throws Exception{long s=e.readRegister("ESP").longValue(),r=read(e,s,4);e.writeRegister("ESP",s+4);if(!e.run(toAddr(r),null,monitor))throw new Exception(e.getLastError());}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  for(int scenario=0;scenario<256;scenario++){
   EmulatorHelper e=new EmulatorHelper(currentProgram);
   try{
    e.writeMemory(toAddr(GROUP),new byte[240*24]);e.writeMemory(toAddr(ROSTER),new byte[240*72]);e.writeMemory(toAddr(UNIT),new byte[240*192]);
    for(int i=0;i<64;i++){e.writeMemory(toAddr(SECT+i*30),new byte[30]);put(e,SECT+i*30,2,65535);}
    for(int i=0;i<4;i++){long s=SECT+i*30;put(e,s,2,i*8);put(e,s+2,2,0);put(e,s+4,2,i*8+8);put(e,s+6,2,8);put(e,s+8,1,i==1?1:0);put(e,s+10,2,(scenario&4)!=0?20+i*10:0);put(e,s+18,1,4+i*8);put(e,s+19,1,4);put(e,s+24,1,(i==2?(scenario&3)*32:0)|(i==0&&(scenario&128)!=0?2:0));put(e,s+25,4,(scenario*37+i*71)%400);}
    for(int i=0;i<8;i++){
     long g=GROUP+i*24,u=UNIT+i*192;put(e,g,4,SECT+30);put(e,g+4,4,-1);put(e,g+8,4,5+((scenario*11+i*23)%150));put(e,g+12,4,-1);put(e,g+16,1,24);put(e,g+17,1,255);put(e,g+21,1,1);
     put(e,ROSTER+i*72+19,1,i);put(e,u+0x31,1,5);put(e,u+0x2c,2,32+i*16);put(e,u+0x2e,2,64);put(e,u+0x7c,1,i<3?1:i==3?16:0);put(e,u+0x30,1,i==0?32:i==1?34:4);put(e,u+0x8e,1,(scenario&8)!=0?3:0);put(e,u+0x9c,1,(scenario&16)!=0&&i==2?4:0);
     if((scenario&32)!=0&&i==7){put(e,g+17,1,4);put(e,g+4,4,SECT+30);put(e,SECT+30+25,4,1);}
    }
    for(long a:new long[]{RET,0x419430,0x411b90})e.setBreakpoint(toAddr(a));put(e,0x4a6813,1,(scenario&64)!=0?1:0);
    invoke(e,0x413c00);invoke(e,0x413da0);invoke(e,0x4143a0);int order=0;
    while(e.readRegister("EIP").longValue()!=RET){long a=e.readRegister("EIP").longValue(),sp=e.readRegister("ESP").longValue();
     if(a==0x411b90)e.writeRegister("EAX",(scenario&8)!=0?1:0);
     else if(a==0x419430){int group=(int)((e.readRegister("EDX").longValue()-UNIT)/192);int sector=(int)((read(e,sp+4,4)-SECT)/30);out.println("{\"kind\":\"order\",\"scenario\":"+scenario+",\"order\":"+order+++",\"group\":"+group+",\"sector\":"+sector+"}");}
     else throw new Exception("Unexpected native task stop");resume(e);
    }
    for(int i=0;i<8;i++){long g=GROUP+i*24;long via=read(e,g+4,4),assigned=read(e,g+12,4);out.println("{\"kind\":\"group\",\"scenario\":"+scenario+",\"group\":"+i+",\"task\":"+(byte)read(e,g+17,1)+",\"via\":"+(via==0xffffffffL?-1:(via-SECT)/30)+",\"assigned\":"+(assigned==0xffffffffL?-1:(assigned-SECT)/30)+"}");}
   }finally{e.dispose();}
  }
 }}
}
