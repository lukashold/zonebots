//5a080→55f10 normal UNIT callback-tail records. Handler/world/order selection
//are explicit call boundaries; raw viewport/visibility/palette writes are native.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.PrintWriter;
public class ZNativeDisplayCallerOracle extends GhidraScript{
 static final long U=0x5e30c8L,G=0x60ec00L,Q=0x62dc50L,S=0x7fff0000L,RET=0x12345000L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(i*8);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 int hash(EmulatorHelper e,int n){int h=0x811c9dc5;if(n>0)for(byte b:e.readMemory(toAddr(Q),n))h=(h^(b&255))*16777619;return h;}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){EmulatorHelper e=new EmulatorHelper(currentProgram);try{
  long handler=currentProgram.getMemory().getInt(toAddr(0x497304))&0xffffffffL;for(long a:new long[]{RET,0x44f1e0,0x454160,handler})e.setBreakpoint(toAddr(a));put(e,0x622cb8,4,0);put(e,0x498c24,2,1024);put(e,0x498c20,2,1024);put(e,0x58aaaa,1,0);put(e,0x586977,1,0);int scenario=0;
  for(int x:new int[]{-32,-31,100,1023,1024})for(int y:new int[]{-48,-47,100,1023,1024})for(int flags69:new int[]{0,4,64,68})for(int remove=0;remove<2;remove++)for(int flags64:new int[]{0,4,5,255}){
   e.writeMemory(toAddr(U),new byte[192]);e.writeMemory(toAddr(G),new byte[72]);e.writeMemory(toAddr(Q),new byte[16]);put(e,0x62b3a4,4,Q);put(e,0x632194,4,G);put(e,U+44,2,x);put(e,U+46,2,y);put(e,U+49,1,4);put(e,U+53,1,1);put(e,U+57,1,3);put(e,U+100,1,flags64);put(e,U+105,1,flags69);put(e,S,4,RET);e.writeRegister("ESP",S);e.writeRegister("ECX",U);int calls=0;
   if(!e.run(toAddr(0x45a080),null,monitor))throw new Exception(e.getLastError());for(int budget=0;e.readRegister("EIP").longValue()!=RET;budget++){if(budget>4)throw new Exception("unexpected callback count");long pc=e.readRegister("EIP").longValue(),sp=e.readRegister("ESP").longValue();if(pc==0x454160)e.writeRegister("EAX",1);else if(pc==handler){calls++;if(remove!=0){put(e,U+49,1,0);put(e,U+48,1,255);}}else if(pc!=0x44f1e0)throw new Exception("unexpected callback "+Long.toHexString(pc));long ret=read(e,sp,4);e.writeRegister("ESP",sp+4);if(!e.run(toAddr(ret),null,monitor))throw new Exception(e.getLastError());}
   int size=(int)(read(e,0x62b3a4,4)-Q),before=hash(e,size);put(e,U+44,2,700);put(e,U+46,2,700);put(e,U+53,1,18);put(e,U+105,1,0);int after=hash(e,size);out.println((scenario++)+","+x+","+y+","+flags69+","+remove+","+flags64+","+calls+","+size+","+(before&0xffffffffL)+","+read(e,U+100,1)+","+(after&0xffffffffL));
  }
 }finally{e.dispose();}}}
}
