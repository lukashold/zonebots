// Complete56640/565a0 query with independently cached buckets and live positions.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeUnitRectangleOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x5e30c8L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){for(int s=0;s<256;s++){
  EmulatorHelper e=new EmulatorHelper(currentProgram);try{e.writeMemory(toAddr(U),new byte[240*192]);e.writeMemory(toAddr(0x635b78),new byte[256+256*32]);int[]count=new int[256];int x=512+s%16*3,y=512+s/16*3;
   for(int i=0;i<240;i++){int px=x-80+i*37%256,py=y-80+i*29%256,b=(py>>3&~15)+(px>>7);if(b>=0&&b<256&&count[b]<32){put(e,0x635c78+b*32+count[b],1,i);put(e,0x635b78+b,1,++count[b]);}put(e,U+i*192+0x2c,2,px+((s+i)%5-2)*40);put(e,U+i*192+0x2e,2,py+((s*3+i)%5-2)*40);put(e,U+i*192+0x7c,1,i%3==0?1:4);}
   e.setBreakpoint(toAddr(RET));e.writeRegister("ESP",S);put(e,S,4,RET);put(e,S+4,2,x+128);put(e,S+8,2,y+128);e.writeRegister("ECX",x);e.writeRegister("EDX",y);if(!e.run(toAddr(0x456640),null,monitor))throw new Exception(e.getLastError());StringBuilder result=new StringBuilder();int n=0;for(;n<288;n++){int slot=(int)read(e,0x62d3a8+n,1);if(slot==255)break;if(n!=0)result.append(',');result.append(slot);}out.println("{\"scenario\":"+s+",\"slots\":["+result+"]}");
  }finally{e.dispose();}
 }}}
}
