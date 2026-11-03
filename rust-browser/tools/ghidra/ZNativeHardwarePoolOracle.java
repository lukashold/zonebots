// Complete581f0 allocator; verify marker and all order-owned bytes in240slots.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeHardwarePoolOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x5e30c8L,H=0x59e6c2L,Q=0x71000000L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 int word(byte[]b,int i){return(short)((b[i]&255)|((b[i+1]&255)<<8));}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){for(int s=0;s<256;s++){
  EmulatorHelper e=new EmulatorHelper(currentProgram);try{e.writeMemory(toAddr(U),new byte[192]);e.writeMemory(toAddr(Q),new byte[44]);byte[]data=new byte[240*44];for(int i=0;i<240;i++){for(int j=0;j<44;j++)data[i*44+j]=(byte)(i*7+j*13+s);data[i*44]=0;data[i*44+1]=0;if(i==s){data[i*44]=-1;data[i*44+1]=-1;}}e.writeMemory(toAddr(H),data);e.setBreakpoint(toAddr(RET));e.writeRegister("ESP",S);put(e,S,4,RET);e.writeRegister("ECX",U);e.writeRegister("EDX",Q);if(!e.run(toAddr(0x4581f0),null,monitor))throw new Exception(e.getLastError());long p=read(e,U+8,4);int selected=p==0xffffffffL?-1:(int)((p-H)/44);byte[]after=e.readMemory(toAddr(H),240*44);int h=0x811c9dc5;for(int i=0;i<240;i++){int o=i*44;int[]values={word(after,o),word(after,o+14),word(after,o+16),after[o+18]&255,after[o+19],after[o+20],after[o+23]&255};for(int n:values)h=(h^n)*16777619;}
   out.println("{\"scenario\":"+s+",\"selected\":"+selected+",\"poolHash\":"+(h&0xffffffffL)+"}");
  }finally{e.dispose();}
 }}}
}
