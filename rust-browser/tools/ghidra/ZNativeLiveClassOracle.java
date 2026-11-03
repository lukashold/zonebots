// Fixed-gun live class, weapon profile and saved passenger class are separate.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.PrintWriter;
public class ZNativeLiveClassOracle extends GhidraScript {
 static final long U=0x5e30c8L,W=0x1100000L,S=0x7fff0000L,RET=0x12345000L;
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 int get(EmulatorHelper e,long a){return e.readMemory(toAddr(a),1)[0]&255;}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  EmulatorHelper e=new EmulatorHelper(currentProgram);
  try{e.setBreakpoint(toAddr(RET));for(int family:new int[]{5,21,22,23})for(int crew:new int[]{0,3,15,16,17,18}){
   e.writeMemory(toAddr(U),new byte[0xc0]);e.writeMemory(toAddr(W),new byte[0x2c]);put(e,U+8,4,W);put(e,U+0x30,1,family);put(e,U+0x7c,1,crew);put(e,U+0xa1,1,crew);put(e,U+0x31,1,21);
   e.writeRegister("ESP",S);put(e,S,4,RET);e.writeRegister("ECX",U);
   if(!e.run(toAddr(0x4582c0),null,monitor))throw new Exception(e.getLastError());
   if(e.readRegister("EIP").longValue()!=RET)throw new Exception("Unexpected live-class stop");
   out.println("{\"family\":"+family+",\"crew\":"+crew+",\"liveClass\":"+get(e,U+0x7c)+",\"profile\":"+get(e,U+0xa1)+",\"savedCrew\":"+get(e,U+0x8e)+",\"state\":"+get(e,U+0x35)+"}");
  }}finally{e.dispose();}
 }}
}
