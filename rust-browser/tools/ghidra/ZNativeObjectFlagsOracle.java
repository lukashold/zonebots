import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.PrintWriter;
public class ZNativeObjectFlagsOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,O=0x1100000L;
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){EmulatorHelper e=new EmulatorHelper(currentProgram);try{
  e.setBreakpoint(toAddr(RET));e.setBreakpoint(toAddr(0x43eb20));for(int scenario=0;scenario<234;scenario++){
   byte[]record=new byte[44];java.util.Arrays.fill(record,(byte)new int[]{0,255,85}[scenario/78]);e.writeMemory(toAddr(O),record);int x=500+scenario*13,y=500+scenario*17;put(e,O,2,x);put(e,O+2,2,y);put(e,0x4d18cc,2,scenario%3);e.writeRegister("ESP",S);put(e,S,4,RET);put(e,S+4,4,x);put(e,S+8,4,y);e.writeRegister("ECX",O);e.writeRegister("EDX",scenario%78);if(!e.run(toAddr(0x43ecf0),null,monitor))throw new Exception(e.getLastError());
   while(e.readRegister("EIP").longValue()!=RET){long pc=e.readRegister("EIP").longValue(),stack=e.readRegister("ESP").longValue();if(pc!=0x43eb20)throw new Exception("Unexpected "+Long.toHexString(pc));e.writeRegister("EAX",0);long ret=read(e,stack,4);e.writeRegister("ESP",stack+4);if(!e.run(toAddr(ret),null,monitor))throw new Exception(e.getLastError());}
   out.println("{\"scenario\":"+scenario+",\"flags19\":"+read(e,O+19,1)+",\"flags13\":"+read(e,O+13,1)+",\"bucket12\":"+read(e,O+12,1)+"}");
  }
 }finally{e.dispose();}}}
}
