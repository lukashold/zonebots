// Complete native405820 callbacks in a flat, unoccupied synthetic world.
//All52 templates retain their real animation, movement and gameplay RNG paths.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeAmbientStepOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,B=0x60b028L,D=0x1000000L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(i*8);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void call(EmulatorHelper e,long f,long cx,long dx)throws Exception{put(e,S,4,RET);e.writeRegister("ESP",S);e.writeRegister("ECX",cx);e.writeRegister("EDX",dx);e.writeRegister("EIP",f);e.getEmulator().setHalt(false);for(int n=0;n<100000;n++){if(e.readRegister("EIP").longValue()==RET)return;if(!e.step(monitor))throw new Exception(e.getLastError());}throw new Exception("CPU budget at "+e.readRegister("EIP").toString(16));}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  for(int template=0;template<52;template++)for(int mode=0;mode<4;mode++){
   EmulatorHelper e=new EmulatorHelper(currentProgram);try{e.writeMemory(toAddr(B),new byte[6200]);for(int j=0;j<100;j++)put(e,B+j*62,2,-1);for(int j=0;j<50;j++)put(e,0x5b00ce+j*10,2,-1);for(int j=0;j<32;j++)put(e,0x60c860+j*60,2,-1);e.writeMemory(toAddr(0x635b78),new byte[256]);e.writeMemory(toAddr(0x58aaaa),new byte[16384]);put(e,0x622cb8,4,0);put(e,0x498c24,2,0);put(e,0x498c20,2,0);put(e,0x62b3a0,4,1);put(e,0x622dc4,1,0);for(int j=0;j<5;j++)put(e,0x622cdc+j*4,4,j+1);
    int[]args={template,127,2,-1,0,(mode&1)==0?0:-1,mode*64,0,0};for(int j=0;j<args.length;j++)put(e,S+4+j*4,4,args[j]);call(e,0x405c60,500,600);
    for(int tick=1;tick<=24;tick++){if(read(e,B,2)!=65535)call(e,0x405820,B,D);long hash=2166136261L;for(byte value:e.readMemory(toAddr(B),62))hash=((hash^(value&255L))*16777619L)&0xffffffffL;StringBuilder line=new StringBuilder(template+","+mode+","+tick+","+hash);for(int j=0;j<5;j++)line.append(',').append(read(e,0x622cdc+j*4,4));out.println(line);out.flush();}
   }finally{e.dispose();}
  }
 }}
}
