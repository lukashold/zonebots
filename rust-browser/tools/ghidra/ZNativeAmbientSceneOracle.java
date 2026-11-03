// Complete405e70 scene initialization. Its separate32-record lava constructor
//is isolated and logs raw arguments;100-target/50-blocker/OBJECT/terrain/RNG
//state follows the actual executable, including multiplayer's zero randoms.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeAmbientSceneOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,B=0x60b028L,O=0x612f80L,D=0x1000000L;long eventHash;int events;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(i*8);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 long hash(EmulatorHelper e,long a,int n){long h=2166136261L;for(byte v:e.readMemory(toAddr(a),n))h=((h^(v&255L))*16777619L)&0xffffffffL;return h;}
 void call(EmulatorHelper e)throws Exception{put(e,S,4,RET);e.writeRegister("ESP",S);e.writeRegister("EIP",0x405e70);e.getEmulator().setHalt(false);for(int n=0;n<500000;n++){long pc=e.readRegister("EIP").longValue();if(pc==RET)return;if(pc==0x405350){long sp=e.readRegister("ESP").longValue();long[]args={e.readRegister("ECX").longValue()&65535,e.readRegister("EDX").longValue()&65535,read(e,sp+4,1),read(e,sp+8,1),read(e,sp+12,1),read(e,sp+16,1)};for(long v:args)eventHash=((eventHash^v)*16777619L)&0xffffffffL;events++;e.writeRegister("ESP",sp+4);e.writeRegister("EIP",read(e,sp,4));continue;}if(!e.step(monitor))throw new Exception(e.getLastError());}throw new Exception("CPU budget");}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  for(int scenario=0;scenario<16;scenario++)for(int world:new int[]{0,2,3})for(int multi=0;multi<2;multi++){
   EmulatorHelper e=new EmulatorHelper(currentProgram);try{e.writeMemory(toAddr(B),new byte[6200]);e.writeMemory(toAddr(O),new byte[8800]);e.writeMemory(toAddr(0x5b00ce),new byte[500]);for(int j=0;j<100;j++)put(e,B+j*62,2,-1);for(int j=0;j<50;j++)put(e,0x5b00ce+j*10,2,-1);for(int j=0;j<200;j++)put(e,O+j*44,2,-1);
    byte[]plane=new byte[16384];for(int j=0;j<plane.length;j++)plane[j]=(byte)(((j+scenario)&1)*128+((scenario&2)!=0&&j%11==0?10:0));e.writeMemory(toAddr(0x58aaaa),plane);
    for(int i=0;i<24;i++){if((i+scenario)%7==0)continue;long r=O+i*44;int x=400+(i*31+scenario*17)%1000,y=400+(i*67+scenario*13)%1000;put(e,r,2,x);put(e,r+2,2,y);put(e,r+4,4,D+i*2);put(e,D+i*2,2,i%3==0?2935+i%5:i%3==1?2285:7000);put(e,r+13,1,128|((i+scenario)&127));}
    put(e,0x62b3a0,4,1);put(e,0x622dc4,1,multi);put(e,0x622c7a,2,0);put(e,0x622cbc,2,0);put(e,0x625eae,1,world);for(int j=0;j<5;j++)put(e,0x622cdc+j*4,4,j+1);events=0;eventHash=2166136261L;call(e);
    long objectHash=2166136261L;for(int i=0;i<200;i++){long r=O+i*44;for(long v:new long[]{read(e,r,2),read(e,r+2,2),read(e,r+13,1)})objectHash=((objectHash^v)*16777619L)&0xffffffffL;}
    StringBuilder line=new StringBuilder(scenario+","+world+","+multi+","+hash(e,B,6200)+","+hash(e,0x5b00ce,500)+","+hash(e,0x58aaaa,16384)+","+objectHash+","+events+","+eventHash);for(int j=0;j<5;j++)line.append(',').append(read(e,0x622cdc+j*4,4));out.println(line);out.flush();
   }finally{e.dispose();}
  }
 }}
}
