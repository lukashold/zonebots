// Execute44a540's target/roster ordinal selection across sparse groups.
import ghidra.app.emulator.EmulatorHelper;
import java.io.PrintWriter;
public class ZNativeIdleTargetOracle extends ZNativeReactionsOracle{
 int selected,calls,seed;
 void invoke(EmulatorHelper e)throws Exception{selected=-1;calls=0;e.writeRegister("ESP",S);put(e,S,4,RET);e.writeRegister("ECX",U);e.writeRegister("EDX",G);e.setBreakpoint(toAddr(RET));if(!e.run(toAddr(0x44a540),null,monitor))throw new Exception(e.getLastError());
  while(e.readRegister("EIP").longValue()!=RET){long pc=e.readRegister("EIP").longValue();if(pc==0x44a3a0){}else if(pc==0x460910){calls++;e.writeRegister("EAX",seed&0xffffffffL);}else if(pc==0x4574c0)selected=e.readRegister("EDX").intValue()&255;else throw new Exception("Unexpected "+Long.toHexString(pc));long stack=e.readRegister("ESP").longValue(),ret=read(e,stack,4);e.writeRegister("ESP",stack+4);if(!e.run(toAddr(ret),null,monitor))throw new Exception(e.getLastError());}
 }
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){EmulatorHelper e=new EmulatorHelper(currentProgram);try{
  for(long a:new long[]{0x44a3a0,0x460910,0x4574c0})e.setBreakpoint(toAddr(a));
  for(int s=0;s<256;s++){e.writeMemory(toAddr(U),new byte[192*32]);e.writeMemory(toAddr(G),new byte[72*2]);put(e,0x632194,4,G);put(e,G+59,1,7);long target=U+7*192;put(e,target+0x30,1,(s&1)!=0?32:4);put(e,target+0x5e,1,(s&2)!=0?0:1);put(e,target+0x31,1,(s&4)!=0?5:1);put(e,target+0x5f,1,1);int count=0;for(int r=0;r<21;r++){int value=(r+s)%4==0?255:r+9;put(e,G+72+19+r,1,value);if(value!=255)count++;}put(e,G+72+18,1,count);seed=s*2027^0x7731a5c9;invoke(e);out.println("{\"scenario\":"+s+",\"target\":"+selected+",\"calls\":"+calls+",\"groupTarget\":"+read(e,G+59,1)+"}");}
 }finally{e.dispose();}}}
}
