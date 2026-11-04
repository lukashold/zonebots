//Actual57ff0 ->41b950. The accepted takeover already owns a W pointer.
import ghidra.app.script.GhidraScript;import ghidra.app.emulator.EmulatorHelper;import java.io.*;
public class ZV47CraneCtor28Oracle extends GhidraScript{
 static final long U=0x1800000L,W=U+256,S=0x7fff0000L,RET=0x12345000L;
 String hex(EmulatorHelper e,long a,int n){StringBuilder s=new StringBuilder();for(byte b:e.readMemory(toAddr(a),n))s.append(String.format("%02x",b&255));return s.toString();}
 void fill(EmulatorHelper e,long a,int n,int seed){byte[]b=new byte[n];for(int i=0;i<n;i++)b[i]=(byte)(seed+i*23);e.writeMemory(toAddr(a),b);}
 public void run()throws Exception{try(PrintWriter o=new PrintWriter(getScriptArgs()[0])){for(int s=0;s<64;s++){EmulatorHelper e=new EmulatorHelper(currentProgram);try{fill(e,U,192,s*17);fill(e,W,44,s*19);fill(e,0x622cdc,20,s*29);String before=hex(e,U,192)+hex(e,W,44)+hex(e,0x622cdc,20);e.writeMemoryValue(toAddr(S),4,RET);e.writeRegister("ESP",S);e.writeRegister("ECX",W);e.writeRegister("EDX",U);e.setBreakpoint(toAddr(RET));e.setBreakpoint(toAddr(0x460910));if(!e.run(toAddr(0x457ff0),null,monitor))throw new Exception(e.getLastError());if(e.readRegister("EIP").longValue()!=RET)throw new Exception("unexpectedRNG");o.println("{\"scenario\":"+s+",\"before\":\""+before+"\",\"after\":\""+hex(e,U,192)+hex(e,W,44)+hex(e,0x622cdc,20)+"\"}");o.flush();}finally{e.dispose();}}}}
}
