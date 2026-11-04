// Executes normal55f10 UNIT and43f260 OBJECT queue producers. Only OBJECT
//lifetime admission/shadow drawing are boundaries; the queue writes are native.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.PrintWriter;
public class ZNativeDisplayProducersOracle extends GhidraScript{
 static final long U=0x5e30c8L,O=0x612f80L,Q=0x62dc50L,S=0x7fff0000L,RET=0x12345000L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(i*8);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 int hash(EmulatorHelper e,long address,int length){int h=0x811c9dc5;if(length>0)for(byte b:e.readMemory(toAddr(address),length))h=(h^(b&255))*16777619;return h;}
 void start(EmulatorHelper e,long address,long ecx,long edx)throws Exception{put(e,S,4,RET);e.writeRegister("ESP",S);e.writeRegister("ECX",ecx);e.writeRegister("EDX",edx);if(!e.run(toAddr(address),null,monitor))throw new Exception(e.getLastError());}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){EmulatorHelper e=new EmulatorHelper(currentProgram);try{
  for(long a:new long[]{RET,0x43f200,0x4666c0,0x41dbf0})e.setBreakpoint(toAddr(a));put(e,0x622cb8,4,0);put(e,0x498c24,2,1024);put(e,0x498c20,2,1024);put(e,0x58aaaa,1,0);put(e,0x4a8c06,1,0);put(e,0x622dc0,1,1);put(e,0x622dc4,1,1);
  int[]phases={0,1,2,3,4,5,6,7,8,16,17,19,20,21,22,23,24,25,26,27,28,29,30,33,42,49,61,62,63,64,65,66,72,73,74};int scenario=0;
  for(int phase:phases)for(int heading=0;heading<8;heading++)for(int elevation:new int[]{-128,-32,0,127})for(int frame:new int[]{-1,0,2,32767}){
   e.writeMemory(toAddr(U),new byte[192]);e.writeMemory(toAddr(Q),new byte[16]);put(e,0x62b3a4,4,Q);put(e,U+44,2,500);put(e,U+46,2,600);put(e,U+48,1,0);put(e,U+50,2,frame);put(e,U+53,1,phase);put(e,U+57,1,heading);int flags=(scenario*37)&255;put(e,U+100,1,flags);put(e,U+105,1,4);put(e,U+118,1,elevation);put(e,U+124,1,phase>=61?15:0);start(e,0x455f10,U,0);if(e.readRegister("EIP").longValue()!=RET)throw new Exception("unexpected UNIT call "+e.readRegister("EIP"));int size=(int)(read(e,0x62b3a4,4)-Q);out.println("u,"+(scenario++)+","+phase+","+heading+","+elevation+","+frame+","+flags+","+size+","+(hash(e,Q,size)&0xffffffffL)+","+read(e,U+100,1));
  }
  scenario=0;for(int kind=0;kind<78;kind++)for(int animation:new int[]{-1,0,4,127})for(int offset:new int[]{-128,0,30,127})for(int flags:new int[]{0,32,34,38,40,48,62,112}){
   long meta=0x498f38L+kind*26;int base=(short)read(e,meta,2);e.writeMemory(toAddr(O),new byte[44]);e.writeMemory(toAddr(Q),new byte[48]);put(e,O,2,500);put(e,O+2,2,600);put(e,O+4,4,meta);put(e,O+13,1,flags);put(e,O+18,1,animation);put(e,O+33,1,offset);start(e,0x43f260,O,Q);for(int budget=0;e.readRegister("EIP").longValue()!=RET;budget++){if(budget>3)throw new Exception("unexpected OBJECT calls");long pc=e.readRegister("EIP").longValue(),sp=e.readRegister("ESP").longValue();if(pc==0x43f200)e.writeRegister("EAX",1);else if(pc!=0x4666c0&&pc!=0x41dbf0)throw new Exception("unexpected OBJECT call "+Long.toHexString(pc));long ret=read(e,sp,4);e.writeRegister("ESP",sp+4);if(!e.run(toAddr(ret),null,monitor))throw new Exception(e.getLastError());}int size=(int)(e.readRegister("EAX").longValue()-Q);if(size<0||size>48)throw new Exception("bad object size"+size);out.println("o,"+(scenario++)+","+kind+","+base+","+(byte)read(e,meta+19,1)+","+animation+","+offset+","+flags+","+size+","+(hash(e,Q,size)&0xffffffffL)+","+(hash(e,Q,Math.min(8,size))&0xffffffffL));
  }
 }finally{e.dispose();}}}
}
