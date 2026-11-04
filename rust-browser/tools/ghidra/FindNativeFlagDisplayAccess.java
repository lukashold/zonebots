// Lists code sites referencing the64 six-byte flag-draw records. The output
//is private evidence; this helper contains no original code or media.
import ghidra.app.script.GhidraScript;
import ghidra.program.model.listing.*;
import ghidra.program.model.scalar.Scalar;
import ghidra.program.model.address.Address;
import java.io.PrintWriter;
public class FindNativeFlagDisplayAccess extends GhidraScript{
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){InstructionIterator it=currentProgram.getListing().getInstructions(true);while(it.hasNext()){Instruction i=it.next();boolean hit=false;for(int op=0;op<i.getNumOperands();op++)for(Object a:i.getOpObjects(op)){long n=a instanceof Scalar?((Scalar)a).getUnsignedValue():a instanceof Address?((Address)a).getOffset():0;if(n>=0x5a1d50&&n<0x5a1ee8)hit=true;}if(hit){Function f=getFunctionContaining(i.getAddress());out.println(i.getAddress()+" "+i+" FUNCTION "+(f==null?"null":f.getEntryPoint()+" "+f.getName()));}}}}
}
