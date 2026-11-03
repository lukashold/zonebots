import ghidra.app.script.GhidraScript;
import java.io.PrintWriter;
public class ZNativeEffectContext extends GhidraScript{
 public void run()throws Exception{try(PrintWriter o=new PrintWriter(getScriptArgs()[0])){
  for(long start:new long[]{0x435e20,0x435e70}){var it=currentProgram.getListing().getInstructions(toAddr(start),true);int n=0;while(it.hasNext()&&n++<55){var i=it.next();o.println(i.getAddress()+" "+i);}}
 }}
}
