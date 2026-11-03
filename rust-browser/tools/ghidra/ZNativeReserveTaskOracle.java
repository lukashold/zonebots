// Original reserve group flag selection00416e10; score/noise and release callbacks
// are intercepted so the chosen raw values and task notifications remain numeric.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeReserveTaskOracle extends GhidraScript{
 static final long RET=0x12345000L,STACK=0x7fff0000L,SECT=0x5c3f0cL,GROUP=0x5c47b8L,UNIT=0x5e30c8L,ROSTER=0x60ec00L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void resume(EmulatorHelper e)throws Exception{long s=e.readRegister("ESP").longValue(),r=read(e,s,4);e.writeRegister("ESP",s+4);if(!e.run(toAddr(r),null,monitor))throw new Exception(e.getLastError());}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  for(int scenario=0;scenario<64;scenario++){
   EmulatorHelper e=new EmulatorHelper(currentProgram);
   try{
    e.writeMemory(toAddr(GROUP),new byte[240*24]);e.writeMemory(toAddr(ROSTER),new byte[240*72]);e.writeMemory(toAddr(UNIT),new byte[240*192]);
    put(e,SECT+24,1,1);put(e,0x5c468d,1,(scenario%4)*30);put(e,0x5c3f08,4,Float.floatToRawIntBits((scenario&4)!=0?100f:0f));put(e,0x4a811c,4,Float.floatToRawIntBits((scenario&8)!=0?200f:0f));
    for(int i=0;i<8;i++){
     long g=GROUP+i*24,u=UNIT+i*192;put(e,g+16,1,(i%3!=0?8:0)|((scenario+i)%2!=0?16:0));put(e,g+21,1,1);put(e,ROSTER+i*72+19,1,i);put(e,u+0x31,1,5);put(e,u+0x7c,1,i%4==0?2:0);put(e,u+0x30,1,i==3?36:4);
    }
    for(long a:new long[]{RET,0x41a800,0x41a8b0,0x417400,0x416d70})e.setBreakpoint(toAddr(a));
    e.writeRegister("ESP",STACK);put(e,STACK,4,RET);if(!e.run(toAddr(0x416e10),null,monitor))throw new Exception(e.getLastError());
    while(e.readRegister("EIP").longValue()!=RET){long a=e.readRegister("EIP").longValue();
     if(a==0x41a800){int i=(int)((e.readRegister("ECX").longValue()-ROSTER)/72);e.writeRegister("EAX",1+(scenario*17+i*23)%170);}
     else if(a==0x41a8b0){} // Leave the pointed-to score unchanged.
     else if(a==0x417400)e.writeRegister("EAX",SECT);
     else if(a==0x416d70){int group=(int)((e.readRegister("ECX").longValue()-GROUP)/24);out.println("{\"kind\":\"release\",\"scenario\":"+scenario+",\"group\":"+group+"}");}
     else throw new Exception("Unexpected reserve task stop");resume(e);
    }
    int budget=(int)Float.intBitsToFloat((int)read(e,0x5c3f00,4));
    for(int i=0;i<8;i++)out.println("{\"kind\":\"flags\",\"scenario\":"+scenario+",\"group\":"+i+",\"value\":"+(int)read(e,0x4a8530+i*4,4)+",\"budget\":"+budget+",\"flags\":"+read(e,GROUP+i*24+16,1)+"}");
   }finally{e.dispose();}
  }
 }}
}
