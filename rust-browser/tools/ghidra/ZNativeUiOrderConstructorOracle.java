// Validated UI target branches441985 and441bf0. UI feedback calls are isolated;
// argument construction executes the original instructions before45ef00.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeUiOrderConstructorOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x5e30c8L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void resume(EmulatorHelper e)throws Exception{long s=e.readRegister("ESP").longValue(),r=read(e,s,4);e.writeRegister("ESP",s+4);if(!e.run(toAddr(r),null,monitor))throw new Exception(e.getLastError());}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){for(int s=0;s<512;s++){
  EmulatorHelper e=new EmulatorHelper(currentProgram);try{boolean hw=(s&1)!=0;int target=s*13%240,actor=(target+1)%240;long a=U+actor*192L,t=U+target*192L;int role=new int[]{-1,0,1}[s/2%3],group=s*19%240;
   e.writeMemory(toAddr(a),new byte[192]);e.writeMemory(toAddr(t),new byte[192]);put(e,t+0x5e,1,role);put(e,t+0x5f,1,group);put(e,0x62dc3c,2,-32000+s*17);
   for(long pc:new long[]{RET,0x45ef00,0x438740,0x441550})e.setBreakpoint(toAddr(pc));e.writeRegister("ESP",S);e.writeRegister("EBP",S+128);put(e,S,4,RET);e.writeRegister("ECX",a);e.writeRegister("EDX",t);e.writeRegister("ESI",a);e.writeRegister("EDI",t);
   if(!e.run(toAddr(hw?0x441bf0:0x441985),null,monitor))throw new Exception(e.getLastError());int n=0;while(e.readRegister("EIP").longValue()!=0x45ef00){if(++n>2)throw new Exception("Too many stops");long pc=e.readRegister("EIP").longValue();if(pc!=0x438740&&pc!=0x441550)throw new Exception("Unexpected stop "+Long.toHexString(pc));resume(e);}
   out.println("{\"scenario\":"+s+",\"opcode\":"+(byte)read(e,0x56ed47,1)+",\"x\":"+(short)read(e,0x62dc00,2)+",\"y\":"+(short)read(e,0x62dc3c,2)+",\"aux\":"+(short)read(e,0x62dc48,2)+"}");
  }finally{e.dispose();}
 }}}
}
