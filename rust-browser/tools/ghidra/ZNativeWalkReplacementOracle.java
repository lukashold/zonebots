// Numeric525a0 frame32/timer3c transfer at a local segment replacement.
import ghidra.app.emulator.EmulatorHelper;
import java.io.PrintWriter;
public class ZNativeWalkReplacementOracle extends ZNativeAnimationInitOracle{
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){EmulatorHelper e=new EmulatorHelper(currentProgram);try{
  int scenario=0;for(int speed=0;speed<13;speed++)for(int frame:new int[]{0,1,2,3,4,255,256,-256})for(int timer:new int[]{-128,-1,0,1,2,7,10,127}){
   e.writeMemory(toAddr(U),new byte[192]);put(e,U+0x2c,2,500);put(e,U+0x2e,2,500);put(e,U+0x35,1,5);put(e,U+0x32,2,frame);put(e,U+0x3c,1,timer);put(e,U+0x64,1,64);put(e,U+0x8b,1,speed);invoke(e,0x4525a0,U,550,550);
   out.println("{\"scenario\":"+scenario+++",\"frame\":"+(short)read(e,U+0x32,2)+",\"timer\":"+(byte)read(e,U+0x3c,1)+",\"phase\":"+read(e,U+0x35,1)+"}");
  }
 }finally{e.dispose();}}}
}
