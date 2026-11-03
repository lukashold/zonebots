// Complete52d90 and52b70, only52500 path destruction isolated and counted.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeOrderHandlersOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x5e30c8L,R=0x60ec00L,H=0x59e6c2L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void resume(EmulatorHelper e)throws Exception{long s=e.readRegister("ESP").longValue(),r=read(e,s,4);e.writeRegister("ESP",s+4);if(!e.run(toAddr(r),null,monitor))throw new Exception(e.getLastError());}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){for(int mode=0;mode<2;mode++)for(int s=0;s<512;s++){
  EmulatorHelper e=new EmulatorHelper(currentProgram);try{e.writeMemory(toAddr(U),new byte[192]);e.writeMemory(toAddr(R),new byte[72]);e.writeMemory(toAddr(H),new byte[44]);put(e,0x632194,4,R);put(e,U+8,4,mode==1&&s%5==0?((s&1)!=0?-1:0):H);put(e,U+0x31,1,s*5);put(e,U+0x9c,1,s*7);put(e,U+0x64,1,s*3);put(e,U+0x71,1,s);put(e,U+0x35,1,7+s%50);put(e,U+0x3c,1,s*11);put(e,U+0x3d,1,s*19);put(e,U+0x61,1,s*13);put(e,U+0x68,1,s*37);put(e,U+0x5c,1,mode==0?4:2);put(e,U+0x50,4,0x1234abcdL+s);put(e,U+0x54,2,-600-s);put(e,U+0x56,2,s&1);put(e,U+0x7c,1,new int[]{0,1,2,4,6}[s%5]);put(e,U+0x30,1,new int[]{5,6,24,25,30,32,34,36,38}[s%9]);put(e,U+6,1,s*11);put(e,R+61,1,s*31);
   put(e,H+7,1,s*13);put(e,H+14,2,500+s);put(e,H+16,2,600-s);put(e,H+18,1,s*17);put(e,H+19,1,s*19);put(e,H+20,1,s*23);put(e,H+23,1,s*29);
   e.setBreakpoint(toAddr(RET));e.setBreakpoint(toAddr(0x452500));e.writeRegister("ESP",S);put(e,S,4,RET);e.writeRegister("ECX",U);if(!e.run(toAddr(mode==0?0x452d90:0x452b70),null,monitor))throw new Exception(e.getLastError());int clear=0;if(e.readRegister("EIP").longValue()==0x452500){clear=1;e.writeRegister("EAX",0);resume(e);}
   out.println("{\"mode\":"+mode+",\"scenario\":"+s+",\"order\":"+(byte)read(e,U+0x5c,1)+",\"flags9c\":"+read(e,U+0x9c,1)+",\"flags31\":"+read(e,U+0x31,1)+",\"flags64\":"+read(e,U+0x64,1)+",\"flags71\":"+read(e,U+0x71,1)+",\"phase\":"+read(e,U+0x35,1)+",\"timer3c\":"+read(e,U+0x3c,1)+",\"timer3d\":"+read(e,U+0x3d,1)+",\"motion61\":"+read(e,U+0x61,1)+",\"field68\":"+read(e,U+0x68,1)+",\"group3d\":"+read(e,R+61,1)+",\"x\":"+(short)read(e,H+14,2)+",\"y\":"+(short)read(e,H+16,2)+",\"selected\":"+read(e,H+18,1)+",\"counter13\":"+(byte)read(e,H+19,1)+",\"counter14\":"+(byte)read(e,H+20,1)+",\"field17\":"+read(e,H+23,1)+",\"clear\":"+clear+"}");
  }finally{e.dispose();}
 }}}
}
