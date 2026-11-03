// Source volcanic32x60 pool. All gameplay helpers/RNG run unmodified;
//the viewport is offscreen and the external500-particle pool is full/disabled.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeLavaOracle extends GhidraScript{
 static final long P=0x60c860L,STACK=0x7fff0000L,RET=0x12345000L;
 void put(EmulatorHelper e,long a,int n,long v){e.writeMemoryValue(toAddr(a),n,v);}
 long word(EmulatorHelper e,long a){byte[]b=e.readMemory(toAddr(a),4);return(b[0]&255L)|((b[1]&255L)<<8)|((b[2]&255L)<<16)|((b[3]&255L)<<24);}
 long hash(byte[]b){long h=2166136261L;for(byte n:b)h=((h^(n&255L))*16777619L)&0xffffffffL;return h;}
 void invoke(EmulatorHelper e,long entry,long cx,long dx)throws Exception{e.writeRegister("ECX",cx);e.writeRegister("EDX",dx);e.writeRegister("ESP",STACK);put(e,STACK,4,RET);if(!e.run(toAddr(entry),null,monitor))throw new Exception(e.getLastError());}
 void setup(EmulatorHelper e,int scenario,int multi){
  e.setBreakpoint(toAddr(RET));e.writeMemory(toAddr(P),new byte[1920]);for(int i=0;i<32;i++)put(e,P+i*60,2,65535);e.writeMemory(toAddr(0x5e30c8),new byte[240*192]);put(e,0x62d5e0,4,1);put(e,0x4d18fe,1,3);put(e,0x622c7a,2,0);put(e,0x622cbc,2,0);put(e,0x625eae,1,3);put(e,0x622dc4,1,multi);put(e,0x622d50,2,1024);put(e,0x622d52,2,768);put(e,0x622e1c,2,0);put(e,0x622cb8,2,10000);put(e,0x622cba,2,10000);put(e,0x498c24,2,320);put(e,0x498c20,2,200);put(e,0x625e3c,2,0);put(e,0x622be4,2,0);
  byte[]terrain=new byte[16384];java.util.Arrays.fill(terrain,(byte)((scenario&1)==0?0:2));e.writeMemory(toAddr(0x58aaaa),terrain);for(int i=0;i<5;i++)put(e,0x622cdc+i*4,4,i==0?1+scenario:i+1);
 }
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(new FileOutputStream(getScriptArgs()[0]))){
  for(int scenario=0;scenario<128;scenario++)for(int multi=0;multi<2;multi++){EmulatorHelper e=new EmulatorHelper(currentProgram);try{setup(e,scenario,multi);int slot=scenario&3;for(int i=0;i<slot;i++){put(e,P+i*60,2,400+i);put(e,P+i*60+24,2,30000);}
   put(e,STACK+4,4,scenario&1);put(e,STACK+8,4,2);put(e,STACK+12,4,2);put(e,STACK+16,4,2);invoke(e,0x405350,100+scenario*3,200+scenario*2);long a=P+slot*60;
   put(e,a+49,1,new int[]{0,8,16,32,64,128,1,3}[scenario%8]);put(e,a+48,1,new int[]{64,68,65,66,72,64,68,68}[(scenario/8)%8]);put(e,a+40,1,scenario*17&255);put(e,a+41,1,scenario%6);put(e,a+42,1,scenario%4);put(e,a+44,1,2+scenario%3);int height=new int[]{0,1,3,12}[(scenario/4)%4];put(e,a+28,2,height);put(e,a+30,2,(height+3)%35);put(e,a+4,2,4);put(e,a+55,1,scenario%12);put(e,a+56,1,6);put(e,a+24,2,scenario%11==0?2:0);
   for(int tick=1;tick<=64;tick++){invoke(e,0x406f80,0,0);out.printf("%d,%d,%d,%d",scenario,multi,tick,hash(e.readMemory(toAddr(P),1920)));for(int i=0;i<5;i++)out.print(","+word(e,0x622cdc+i*4));out.println();}
  }finally{e.dispose();}}
 }}
}
