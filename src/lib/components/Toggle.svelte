<!-- Interrupteur marche/arrêt accessible (case à cocher avec le rôle « switch »). -->
<script lang="ts">
  interface Props {
    label: string;
    checked: boolean;
    disabled?: boolean;
    onchange: (checked: boolean) => void;
  }

  let { label, checked, disabled = false, onchange }: Props = $props();
</script>

<label class="toggle">
  <span>{label}</span>
  <input
    type="checkbox"
    role="switch"
    {checked}
    {disabled}
    onchange={(event) => onchange(event.currentTarget.checked)}
  />
</label>

<style>
  .toggle {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    padding: 0.5rem 0;
    cursor: pointer;
  }

  input {
    appearance: none;
    flex-shrink: 0;
    position: relative;
    width: 2.5rem;
    height: 1.4rem;
    margin: 0;
    border-radius: 999px;
    background: var(--color-border);
    cursor: pointer;
    transition: background-color 0.15s;
  }

  input::after {
    content: "";
    position: absolute;
    top: 0.2rem;
    left: 0.2rem;
    width: 1rem;
    height: 1rem;
    border-radius: 50%;
    background: var(--color-text);
    transition: transform 0.15s;
  }

  input:checked {
    background: var(--color-accent);
  }

  input:checked::after {
    transform: translateX(1.1rem);
  }

  input:focus-visible {
    outline: 2px solid var(--color-accent);
    outline-offset: 2px;
  }

  input:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
