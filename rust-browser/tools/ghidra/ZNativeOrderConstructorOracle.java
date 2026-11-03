// Actual15ee0 and150a0 construction, stopping before45ef00 dispatch.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeOrderConstructorOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x5e30c8L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){for(int s=0;s<512;s++){
  EmulatorHelper e=new EmulatorHelper(currentProgram);try{int target=s*13%240,actor=(target+1)%240;long a=U+actor*192L,t=U+target*192L;int cls=new int[]{0,1,2,3,4,5,6,15}[s%8],family=s%5==0?36:4,role=new int[]{-1,0,1}[s/8%3],group=s*19%240;
   e.writeMemory(toAddr(a),new byte[192]);e.writeMemory(toAddr(t),new byte[192]);put(e,a+0x7c,1,cls);put(e,a+0x30,1,family);put(e,t+0x5e,1,role);put(e,t+0x5f,1,group);
   put(e,0x56ed47,1,77);put(e,0x62dc00,2,-1900+s);put(e,0x62dc3c,2,-32700+s*13);put(e,0x62dc48,2,3900+s);put(e,0x62dc04,4,0);
   e.setBreakpoint(toAddr(RET));e.setBreakpoint(toAddr(0x45ef00));e.writeRegister("ESP",S);put(e,S,4,RET);put(e,S+4,2,-300+s*11);e.writeRegister("ECX",t);e.writeRegister("EDX",a);
   if(!e.run(toAddr(0x415ee0),null,monitor))throw new Exception(e.getLastError());boolean called=e.readRegister("EIP").longValue()==0x45ef00;
   out.println("{\"scenario\":"+s+",\"called\":"+(called?1:0)+",\"opcode\":"+(byte)read(e,0x56ed47,1)+",\"x\":"+(short)read(e,0x62dc00,2)+",\"y\":"+(short)read(e,0x62dc3c,2)+",\"aux\":"+(short)read(e,0x62dc48,2)+"}");
  }finally{e.dispose();}
 }}}
}
