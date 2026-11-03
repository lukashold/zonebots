// Execute the fort-only tactical wrapper and its entry-path predicate.
// Results contain fixture identifiers and numerical decisions only.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeFortTaskOracle extends GhidraScript {
 static final long RET=0x12345000L, STACK=0x7fff0000L, GROUP=0x5c47b8L, SECT=0x5c3f0cL, ROSTER=0x60ec00L, UNIT=0x5e30c8L, PATH=0x70000000L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void invoke(EmulatorHelper e,long a,long g)throws Exception{e.writeRegister("ESP",STACK);put(e,STACK,4,RET);e.writeRegister("ECX",g);if(!e.run(toAddr(a),null,monitor))throw new Exception(e.getLastError());}
 void resume(EmulatorHelper e)throws Exception{long s=e.readRegister("ESP").longValue(),r=read(e,s,4);e.writeRegister("ESP",s+4);if(!e.run(toAddr(r),null,monitor))throw new Exception(e.getLastError());}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  for(int scenario=0;scenario<32;scenario++){
   EmulatorHelper e=new EmulatorHelper(currentProgram);
   try{
    e.writeMemory(toAddr(GROUP),new byte[240*24]);e.writeMemory(toAddr(SECT),new byte[4*30]);
    for(int i=0;i<4;i++)put(e,SECT+i*30+24,1,i);
    for(int i=0;i<8;i++){long g=GROUP+i*24;put(e,g,4,(scenario&16)!=0&&i==7?0xffffffffL:SECT+((i+scenario)%4)*30);put(e,g+16,1,(i+scenario)%4*8);put(e,g+21,1,(scenario&8)==0||i%2==0?1:0);}
    for(long a:new long[]{RET,0x413670,0x4139e0})e.setBreakpoint(toAddr(a));
    invoke(e,0x413b50,0);
    while(e.readRegister("EIP").longValue()!=RET){long a=e.readRegister("EIP").longValue();int group=(int)((e.readRegister("ECX").longValue()-GROUP)/24);
     if(a==0x413670)e.writeRegister("EAX",((scenario+group)&4)!=0?1:0);
     else if(a==0x4139e0)out.println("{\"kind\":\"capture\",\"scenario\":"+scenario+",\"group\":"+group+"}");
     else throw new Exception("Unexpected fort wrapper stop");resume(e);
    }
   }finally{e.dispose();}
  }
  for(int scenario=0;scenario<64;scenario++){
   EmulatorHelper e=new EmulatorHelper(currentProgram);
   try{
    e.writeMemory(toAddr(ROSTER),new byte[240*72]);e.writeMemory(toAddr(UNIT),new byte[240*192]);
    int fx=(scenario&32)!=0?32760:1000,fy=(scenario&32)!=0?-32760:1000;
    put(e,0x622d58,2,fx);put(e,0x622d5a,2,fy);e.setBreakpoint(toAddr(RET));
    for(int i=0;i<8;i++){
     long r=ROSTER+i*72,u=UNIT+i*192,p=PATH+i*8;
     put(e,r+0x3a,1,(scenario&1)!=0?1:0);put(e,r+0x2c,2,fx+80+(i==1?1:0));put(e,r+0x2e,2,fy+72+(i==2?1:0));put(e,r+0x13,1,(scenario&2)!=0&&i==3?255:i);
     put(e,u+0x31,1,(scenario&4)!=0&&i==4?0:1);put(e,u+0x14,4,(scenario&8)!=0&&i==5?0:(scenario&16)!=0&&i==6?0xffffffffL:p);
     put(e,p,2,fx+80+(i==0?1:0));put(e,p+2,2,fy+(i%3==0?72:i%3==1?-48:73));put(e,0x4a6812,1,0);
     invoke(e,0x413670,GROUP+i*24);
     out.println("{\"kind\":\"entering\",\"scenario\":"+scenario+",\"group\":"+i+",\"result\":"+(e.readRegister("EAX").intValue()&255)+",\"shared\":"+read(e,0x4a6812,1)+"}");
    }
   }finally{e.dispose();}
  }
 }}
}
