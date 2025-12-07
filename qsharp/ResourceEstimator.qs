namespace Sentinel.Azure {
    open Sentinel.Quantum;
    open Microsoft.Quantum.Intrinsic;

    /// # Summary
    /// Entry Point for Azure Quantum Resource Estimator.
    /// Calculates Physical Qubits, Runtime, and T-Factories required for the Pricing Oracle.
    @EntryPoint()
    operation EstimateResources() : Double {
        // Run with standard precision
        return RunPricingEstimation(4);
    }
}
