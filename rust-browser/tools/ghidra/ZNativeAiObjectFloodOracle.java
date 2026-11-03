// The OBJECT constructor uses the same terrain flood as sector initialization,
// but preserves the object's exact sub-tile coordinates for sector lookup.
import ghidra.app.emulator.EmulatorHelper;
import java.io.PrintWriter;
public class ZNativeAiObjectFloodOracle extends ZNativeAiTerrainOracle {
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  for(int scenario=0;scenario<32;scenario++){
   EmulatorHelper e=new EmulatorHelper(currentProgram);
   try{e.writeMemory(toAddr(0x58aaaa),terrain(scenario));put(e,0x474078,4,MALLOC);put(e,0x474058,4,FREE);
    for(long a:new long[]{MALLOC,FREE,0x417400})e.setBreakpoint(toAddr(a));long heap=0x1100000;
    e.writeRegister("EDX",1024+(scenario*3)%16);invoke(e,0x43eb20,1024+scenario%16);
    while(e.readRegister("EIP").longValue()!=RET){long a=e.readRegister("EIP").longValue(),s=e.readRegister("ESP").longValue();
     if(a==MALLOC){int size=(int)read(e,s+4,4);e.writeMemory(toAddr(heap),new byte[size]);e.writeRegister("EAX",heap);heap+=size+0x100;}
     else if(a==FREE)e.writeRegister("EAX",0);
     else if(a==0x417400){int x=(short)e.readRegister("ECX").intValue();e.writeRegister("EAX",scenario%8==5&&x>1024?B+30:B);}
     else throw new Exception("Unexpected object flood stop");resume(e);
    }
    out.println("{\"scenario\":"+scenario+",\"result\":"+(e.readRegister("EAX").intValue()&255)+"}");
   }finally{e.dispose();}
  }
 }}
}
