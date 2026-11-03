// Real4e9b0 and17400;4e910 is the explicit follow-up boundary.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeGroupFlagCleanupOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x5e30c8L,B=0x5c3f0cL;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void resume(EmulatorHelper e)throws Exception{long s=e.readRegister("ESP").longValue(),r=read(e,s,4);e.writeRegister("ESP",s+4);if(!e.run(toAddr(r),null,monitor))throw new Exception(e.getLastError());}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){for(int s=0;s<32;s++){
  EmulatorHelper e=new EmulatorHelper(currentProgram);try{e.writeMemory(toAddr(U),new byte[240*192]);e.writeMemory(toAddr(B),new byte[64*30]);for(int i=0;i<64;i++)put(e,B+i*30,2,-1);for(int i=0;i<2;i++){put(e,B+i*30,2,i*10);put(e,B+i*30+2,2,0);put(e,B+i*30+4,2,i==0?10:30);put(e,B+i*30+6,2,20);}
   for(int i=0;i<240;i++){long u=U+i*192;put(e,u+0x31,1,(i+s)%4|((i+s)%5!=0?4:0));put(e,u+0x8a,1,0xa0|((i+s)%2==0?4:0));put(e,u+0x2c,2,(i+s)%3==0?400:80);put(e,u+0x2e,2,80);}
   long actor=U+239*192;put(e,actor+0x31,1,4|s%4);put(e,actor+0x7c,1,s%3);put(e,actor+0x2c,2,(s&8)!=0?-32:(s&4)!=0?400:80);e.setBreakpoint(toAddr(RET));e.setBreakpoint(toAddr(0x44e910));e.writeRegister("ESP",S);put(e,S,4,RET);e.writeRegister("ECX",actor);if(!e.run(toAddr(0x44e9b0),null,monitor))throw new Exception(e.getLastError());
   int joins=0,jh=0x811c9dc5,fh=0x811c9dc5;while(e.readRegister("EIP").longValue()!=RET){if(++joins>240)throw new Exception("too many joins");int slot=(e.readRegister("ECX").intValue()-(int)U)/192;jh=(jh^slot)*16777619;resume(e);}for(int i=0;i<240;i++)fh=(fh^(int)read(e,U+i*192+0x8a,1))*16777619;
   out.println("{\"scenario\":"+s+",\"flags\":"+(fh&0xffffffffL)+",\"joins\":"+joins+",\"joinHash\":"+(jh&0xffffffffL)+"}");
  }finally{e.dispose();}
 }}}
}
