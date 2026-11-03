// Complete native405820 callbacks in a synthetic contact worlds; external water/mark actions isolated.
//All52 templates retain their real animation, movement and gameplay RNG paths.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeAmbientContactOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,B=0x60b028L,D=0x1000000L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(i*8);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void call(EmulatorHelper e,long f,long cx,long dx)throws Exception{put(e,S,4,RET);e.writeRegister("ESP",S);e.writeRegister("ECX",cx);e.writeRegister("EDX",dx);e.writeRegister("EIP",f);e.getEmulator().setHalt(false);for(int n=0;n<100000;n++){if(e.readRegister("EIP").longValue()==RET)return;long pc=e.readRegister("EIP").longValue();if(pc==0x47326a){long sp=e.readRegister("ESP").longValue(),dst=read(e,sp+4,4),src=read(e,sp+8,4),len=read(e,sp+12,4);if(len>4096)throw new Exception("memcpy bounds");e.writeMemory(toAddr(dst),e.readMemory(toAddr(src),(int)len));e.writeRegister("ESP",sp+4);e.writeRegister("EAX",dst);e.writeRegister("EIP",read(e,sp,4));continue;}if(pc==0x41dcc0||pc==0x4057d0){long sp=e.readRegister("ESP").longValue();put(e,D+0x200+(pc==0x41dcc0?0:4),4,read(e,D+0x200+(pc==0x41dcc0?0:4),4)+1);e.writeRegister("ESP",sp+4);e.writeRegister("EIP",read(e,sp,4));continue;}if(!e.step(monitor))throw new Exception(e.getLastError());}throw new Exception("CPU budget at "+e.readRegister("EIP").toString(16));}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  for(int template=0;template<52;template++)for(int mode=0;mode<12;mode++){
   EmulatorHelper e=new EmulatorHelper(currentProgram);try{e.writeMemory(toAddr(B),new byte[6200]);for(int j=0;j<100;j++)put(e,B+j*62,2,-1);for(int j=0;j<50;j++)put(e,0x5b00ce+j*10,2,-1);for(int j=0;j<32;j++)put(e,0x60c860+j*60,2,-1);e.writeMemory(toAddr(0x635b78),new byte[256]);e.writeMemory(toAddr(0x58aaaa),new byte[16384]);put(e,0x622cb8,4,0);put(e,0x498c24,2,0);put(e,0x498c20,2,0);put(e,0x62b3a0,4,1);put(e,0x622dc4,1,0);for(int j=0;j<5;j++)put(e,0x622cdc+j*4,4,j+1);
    int[]args={template,127,2,-1,0,(mode&1)==0?0:-1,0,0,0};for(int j=0;j<args.length;j++)put(e,S+4+j*4,4,args[j]);call(e,0x405c60,500,600);
    if(mode<3){byte[]counts=new byte[256];java.util.Arrays.fill(counts,(byte)1);e.writeMemory(toAddr(0x635b78),counts);e.writeMemory(toAddr(0x635c78),new byte[8192]);put(e,0x5e30c8+0x2c,2,500);put(e,0x5e30c8+0x2e,2,600);put(e,0x5e30c8+0x7c,1,mode==0?1:0);put(e,0x5e30c8+0x35,1,mode==1?18:2);}
    if(mode==3||mode==4){put(e,B+57,1,mode==3?1:2);put(e,B+45,1,1);put(e,B+53,1,3);}
    if(mode==5)put(e,B+38,2,2);
    if(mode==6)put(e,B+52,1,read(e,B+52,1)|16);
    if(mode==7)put(e,B+47,1,0);
    if(mode==8)put(e,B+53,1,16);
    if(mode==9){put(e,B+52,1,read(e,B+52,1)|32);put(e,B+55,1,1);}
    if(mode==10){byte[]map=new byte[16384];java.util.Arrays.fill(map,(byte)9);e.writeMemory(toAddr(0x58aaaa),map);}
    if(mode==11){for(int j=0;j<50;j++){put(e,0x5b00ce+j*10,2,496+j%9);put(e,0x5b00ce+j*10+2,2,596+j/9);}}

    for(int tick=1;tick<=24;tick++){if(read(e,B,2)!=65535)call(e,0x405820,B,D);long hash=2166136261L;for(byte value:e.readMemory(toAddr(B),62))hash=((hash^(value&255L))*16777619L)&0xffffffffL;StringBuilder line=new StringBuilder(template+","+mode+","+tick+","+hash);for(int j=0;j<5;j++)line.append(',').append(read(e,0x622cdc+j*4,4));line.append(',').append(read(e,D+0x200,4)).append(',').append(read(e,D+0x204,4));out.println(line);out.flush();}
   }finally{e.dispose();}
  }
 }}
}
