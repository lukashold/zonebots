// Execute native candidate collection, ordering, and initial sector allocation.
// Final move dispatch is intercepted; no instruction bytes are exported.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeCaptureTaskOracle extends GhidraScript{
 static final long RET=0x12345000L,STACK=0x7fff0000L,SECT=0x5c3f0cL,GROUP=0x5c47b8L,UNIT=0x5e30c8L,ROSTER=0x60ec00L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void invoke(EmulatorHelper e,long a)throws Exception{e.writeRegister("ESP",STACK);put(e,STACK,4,RET);e.writeRegister("ECX",SECT);if(!e.run(toAddr(a),null,monitor))throw new Exception(e.getLastError());}
 void resume(EmulatorHelper e)throws Exception{long s=e.readRegister("ESP").longValue(),r=read(e,s,4);e.writeRegister("ESP",s+4);if(!e.run(toAddr(r),null,monitor))throw new Exception(e.getLastError());}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  for(int scenario=0;scenario<64;scenario++){
   EmulatorHelper e=new EmulatorHelper(currentProgram);
   try{
    e.writeMemory(toAddr(GROUP),new byte[240*24]);e.writeMemory(toAddr(ROSTER),new byte[240*72]);e.writeMemory(toAddr(UNIT),new byte[240*192]);
    for(int i=0;i<64;i++){e.writeMemory(toAddr(SECT+i*30),new byte[30]);put(e,SECT+i*30,2,65535);}
    long s=SECT;put(e,s,2,0);put(e,s+2,2,0);put(e,s+4,2,8);put(e,s+6,2,8);put(e,s+8,1,0);put(e,s+18,1,4);put(e,s+19,1,4);put(e,s+24,1,(scenario&32)!=0?2:(scenario&3)*32);put(e,s+25,4,100);
    put(e,0x622d58,2,1000);put(e,0x622d5a,2,1000);
    for(int i=0;i<8;i++){
     long g=GROUP+i*24;int count=1+(scenario+i)%3;put(e,g,4,SECT);put(e,g+4,4,SECT);put(e,g+8,4,200);put(e,g+12,4,SECT);put(e,g+16,1,24);put(e,g+17,1,(scenario&2)!=0&&i==3?3:(scenario&16)!=0&&i==7?1:0);put(e,g+21,1,1);
     for(int m=0;m<21;m++)put(e,ROSTER+i*72+19+m,1,m<count?i*3+m:255);
     for(int m=0;m<count;m++){
      long u=UNIT+(i*3+m)*192;put(e,u+0x31,1,5|((scenario&4)!=0&&i==4&&m==0?32:0)|((scenario&1)!=0&&i==2&&m==count-1?64:0));put(e,u+0x2c,2,64);put(e,u+0x2e,2,64);put(e,u+0x7c,1,i<2?1:0);put(e,u+0x30,1,i==0?32:i==1?34:4);put(e,u+0x8e,1,(scenario&8)!=0?3:0);put(e,u+0x5e,1,m==0?255:m);put(e,u+0x5f,1,i);put(e,u+0x8a,1,(scenario&8)!=0&&i==3&&m==count-1?4:0);put(e,u+0x9c,1,(scenario&32)!=0&&i==6?4:0);
     }
    }
    for(long a:new long[]{RET,0x454070,0x4540b0,0x459c60,0x452810,0x413670,0x411b90})e.setBreakpoint(toAddr(a));
    int direct=0;
    for(int n=0;n<8;n++){
     long g=GROUP+n*24;put(e,0x632194,4,ROSTER+n*72);
     e.writeRegister("ESP",STACK);put(e,STACK,4,RET);e.writeRegister("ECX",g);if(!e.run(toAddr(0x4139e0),null,monitor))throw new Exception(e.getLastError());
     while(e.readRegister("EIP").longValue()!=RET){long a=e.readRegister("EIP").longValue(),sp=e.readRegister("ESP").longValue();
      if(a==0x454070)e.writeRegister("EAX",1);
      else if(a==0x4540b0||a==0x452810)e.writeRegister("EAX",0);
      else if(a==0x413670)e.writeRegister("EAX",(scenario&16)!=0?1:0);
      else if(a==0x411b90)e.writeRegister("EAX",(scenario&8)!=0?1:0);
      else if(a==0x459c60){int member=(int)((e.readRegister("ECX").longValue()-UNIT)/192);int group=(int)read(e,UNIT+member*192+0x5f,1);int x=e.readRegister("EDX").intValue()&65535,y=(int)read(e,sp+4,4)&65535;out.println("{\"kind\":\"direct\",\"scenario\":"+scenario+",\"order\":"+direct+++",\"group\":"+group+",\"member\":"+member+",\"x\":"+x+",\"y\":"+y+"}");e.writeRegister("EAX",0);}
      else throw new Exception("Unexpected personal capture stop");resume(e);
     }
    }
    for(int i=0;i<8;i++){long g=GROUP+i*24;long via=read(e,g+4,4),assigned=read(e,g+12,4);out.println("{\"kind\":\"group\",\"scenario\":"+scenario+",\"group\":"+i+",\"task\":"+(byte)read(e,g+17,1)+",\"via\":"+(via==0xffffffffL?-1:(via-SECT)/30)+",\"assigned\":"+(assigned==0xffffffffL?-1:(assigned-SECT)/30)+"}");}
   }finally{e.dispose();}
  }
 }}
}
