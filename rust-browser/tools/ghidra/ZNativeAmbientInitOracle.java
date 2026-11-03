// Native100x62 ambient-target constructor: all52 templates, signed inputs,
// recycled record bytes, owner/delay flags and first-free allocation.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeAmbientInitOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,B=0x60b028L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(i*8);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void call(EmulatorHelper e,int x,int y)throws Exception{put(e,S,4,RET);e.writeRegister("ESP",S);e.writeRegister("ECX",x&65535);e.writeRegister("EDX",y&65535);e.writeRegister("EIP",0x405c60);e.getEmulator().setHalt(false);for(int n=0;n<20000;n++){if(e.readRegister("EIP").longValue()==RET)return;if(!e.step(monitor))throw new Exception(e.getLastError());}throw new Exception("CPU budget");}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  for(int template=0;template<52;template++)for(int mode=0;mode<16;mode++){
   EmulatorHelper e=new EmulatorHelper(currentProgram);try{int free=mode%5;byte[]pool=new byte[6200];for(int j=0;j<pool.length;j++)pool[j]=(byte)(j*13+mode*7);e.writeMemory(toAddr(B),pool);for(int j=0;j<100;j++)put(e,B+j*62,2,j<free?32+j:-1);put(e,0x62b3a0,4,(mode&8)==0?1:0);
    int[]args={template,mode*9+1,mode*7+1,(mode&1)==0?-1:3,mode*5+2,(mode&2)==0?-1:0,mode*37,mode*7-20,mode*11};for(int j=0;j<args.length;j++)put(e,S+4+j*4,4,args[j]);call(e,(mode&4)==0?300:-300,(mode&2)==0?400:-400);
    long hash=2166136261L;for(byte value:e.readMemory(toAddr(B+free*62),62))hash=((hash^(value&255L))*16777619L)&0xffffffffL;out.println(template+","+mode+","+free+","+hash);out.flush();
   }finally{e.dispose();}
  }
 }}
}
