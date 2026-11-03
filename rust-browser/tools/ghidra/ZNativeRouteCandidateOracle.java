// Native local-route selection0043e190 and completed packed-cell decoding.
// World adjustment/road replacement helpers are intercepted at explicit gates.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeRouteCandidateOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x1000000L,N=0x5f8428L,C=0x4d15f0L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(i*8);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void invoke(EmulatorHelper e)throws Exception{e.writeRegister("ESP",S);put(e,S,4,RET);e.writeRegister("ECX",U);if(!e.run(toAddr(0x43e190),null,monitor))throw new Exception(e.getLastError());}
 void resume(EmulatorHelper e)throws Exception{long s=e.readRegister("ESP").longValue(),r=read(e,s,4);e.writeRegister("ESP",s+4);if(!e.run(toAddr(r),null,monitor))throw new Exception(e.getLastError());}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  for(int fixture=0;fixture<4096;fixture++){
   EmulatorHelper e=new EmulatorHelper(currentProgram);try{e.writeMemory(toAddr(U),new byte[192]);e.writeMemory(toAddr(C),new byte[400]);put(e,U+0x31,1,4);put(e,0x586862,1,0);int f=fixture;int[]prices={0,10,254,255};for(int i=0;i<4;i++){int v=f&7;f>>=3;put(e,C+i*100,1,prices[v&3]);put(e,C+i*100+1,1,(v&4)!=0?-2:0);}for(long a:new long[]{RET,0x43e0b0,0x43dd50,0x43dcf0})e.setBreakpoint(toAddr(a));invoke(e);int selected=-1;if(e.readRegister("EIP").longValue()==0x43e0b0)selected=(e.readRegister("EDX").intValue()-(int)C)/100;out.println("{\"kind\":\"choice\",\"fixture\":"+fixture+",\"selected\":"+selected+"}");}finally{e.dispose();}
  }
  for(int vehicle=0;vehicle<2;vehicle++)for(int packed:new int[]{0,1,127,128,129,255,256,4095,8191,16383}){
   EmulatorHelper e=new EmulatorHelper(currentProgram);try{e.writeMemory(toAddr(U),new byte[192]);e.writeMemory(toAddr(C),new byte[400]);e.writeMemory(toAddr(N),new byte[16]);put(e,U+0x31,1,4);put(e,U+0x7c,1,vehicle);put(e,U+0x5e,1,0);put(e,0x63288c,4,U);put(e,C,1,1);put(e,C+1,1,-1);put(e,C+2,2,packed);put(e,N+4,4,-1);for(long a:new long[]{RET,0x43e0b0,0x43c1b0,0x43df20})e.setBreakpoint(toAddr(a));invoke(e);
    while(e.readRegister("EIP").longValue()!=RET){if(e.readRegister("EIP").longValue()==0x43df20)e.writeRegister("EAX",0);resume(e);}
    out.println("{\"kind\":\"decode\",\"vehicle\":"+vehicle+",\"packed\":"+packed+",\"x\":"+(short)read(e,N,2)+",\"y\":"+(short)read(e,N+2,2)+"}");
   }finally{e.dispose();}
  }
 }}
}
