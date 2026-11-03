// Executes native occupied APC -> parked object conversion00458af0/0044de00.
// Only object allocation and its collision predicate are intercepted.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeApcParkingOracle extends GhidraScript {
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x1000000L,W=U+0x1000,O=U+0x2000;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void resume(EmulatorHelper e)throws Exception{long s=e.readRegister("ESP").longValue(),r=read(e,s,4);e.writeRegister("ESP",s+4);if(!e.run(toAddr(r),null,monitor))throw new Exception(e.getLastError());}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  for(int x:new int[]{492,499,500,503,504,505,510,511,512,518,519,520,527})for(int hp:new int[]{0,1,24,25,49,70,200,299,300}){
   EmulatorHelper e=new EmulatorHelper(currentProgram);
   try{
    e.writeMemory(toAddr(U),new byte[0x3000]);e.writeMemory(toAddr(0x5e30c8),new byte[240*0xc0]);e.writeMemory(toAddr(0x58aaaa),new byte[16384]);put(e,U+8,4,W);put(e,U+0x30,1,32);put(e,U+0x7c,1,1);put(e,U+0x2c,2,x);put(e,U+0x2e,2,600);put(e,W+8,2,hp);put(e,W+10,2,200);put(e,0x4a6e77,1,1);put(e,0x622dc4,1,1);
    for(long a:new long[]{RET,0x458a40,0x44dd70})e.setBreakpoint(toAddr(a));e.writeRegister("ESP",S);put(e,S,4,RET);e.writeRegister("ECX",U);if(!e.run(toAddr(0x458af0),null,monitor))throw new Exception(e.getLastError());
    while(e.readRegister("EIP").longValue()!=RET){long a=e.readRegister("EIP").longValue(),s=e.readRegister("ESP").longValue();if(a==0x458a40){put(e,read(e,s+12,4),4,O);put(e,O,2,e.readRegister("EDX").longValue());put(e,O+2,2,read(e,s+4,4));put(e,O+4,4,0x49925e);e.writeRegister("EAX",0);}else if(a==0x44dd70)e.writeRegister("EAX",0);else throw new Exception("Unexpected parking stop");resume(e);}
    out.println("{\"x\":"+x+",\"y\":600,\"hp\":"+hp+",\"parkedX\":"+read(e,O,2)+",\"parkedY\":"+read(e,O+2,2)+",\"parkedHp\":"+(read(e,O+15,1)*25)+"}");
   }finally{e.dispose();}
  }
 }}
}
